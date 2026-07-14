use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use syn::{DeriveInput, parse_macro_input};
use quote::{format_ident, quote};
use darling::{FromDeriveInput, FromField};

/*
#[derive(BindableStruct)]
#[visibility(FRAGMENT)] // Sets the default visibility of the entire struct
struct MyBindGroup {
    #[binding(0)] // Sets the binding of the resource, can only be placed on Bindables
    #[visibility(COMPUTE)] // Adds to the struct's default visibility just for this field
    pub buffer: BindableBuffer<f32, BufStorage<false>>,
    #[binding(1)]
    pub texture: BindableTexture<TexStorage<Rgba32Float, ReadWrite, D2>>,
    #[binding(2)]
    pub sampler: BindableSampler<Filtering>
}*/


#[proc_macro_derive(BindableStruct, attributes(visibility, binding))]
pub fn bindable_struct_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let opts = match StructOpts::from_derive_input(&input) {
        Ok(o) => o,
        Err(e) => return e.write_errors().into(),
    };

    expand(opts)
        .unwrap_or_else(|e| e.write_errors())
        .into()
}

const BINDABLE_TYPES: &[&str] = &[
    "BindableBuffer", "BindableTexture", "BindableSampler",
    "BindableBufferVector", "BindableTextureArray", "BindableSamplerArray"
];

fn expand(opts: StructOpts) -> Result<TokenStream2, darling::Error> {
    let crate_path = match crate_name("wgpu-bindutils").or_else(|_| crate_name("wgpu-bindutils-core")) {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        },
        Err(_) => return Err(darling::Error::custom("wgpu-bindutils-core not found in Cargo.toml")),
    };

    let wgpu_path = match crate_name("wgpu") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        },
        Err(_) => quote!(::wgpu)
    };

    

    let name = &opts.ident;
    let name_str = name.to_string();
    let (impl_generics, ty_generics, where_clause) = opts.generics.split_for_impl();

    let mut misses_default_visibility = false;
    let default_visibility = opts
        .get_visibility()?
        .clone()
        .unwrap_or_else(|| {
            misses_default_visibility = true;
            syn::parse_quote!(#wgpu_path::ShaderStages::NONE)
        });

    let fields = opts
        .data
        .take_struct()
        .ok_or_else(|| darling::Error::custom("BindableStruct only supports structs"))?
        .fields;

    // Check that:
    //     1) All BindableFields have at least the #[binding] attribute
    //     2) No non-BindableField can have the #[binding] or #[visibility] attributes
    // -------------------------------------------------------------------------------
    let mut errors: Vec<darling::Error> = vec![];
    let mut bindable_fields = vec![];

    for field in &fields {
        let looks_bindable = matches!(
            &field.ty,
            syn::Type::Path(tp)
                if tp.path.segments.last().map_or(false, |s| BINDABLE_TYPES.contains(&&s.ident.to_string().as_str()))
        );

        let has_binding = field.get_binding()?.is_some();
        let has_visibility = field.get_visibility()?.is_some();

        if !looks_bindable {
            if has_binding {
                errors.push(darling::Error::custom("#[binding] can only be applied to a BindableField").with_span(&field.ty));
            }
            if has_visibility {
                errors.push(darling::Error::custom("#[visibility] can only be applied to a BindableField").with_span(&field.ty));
            }
        } else if !has_binding {
            errors.push(darling::Error::custom("A BindableField must have explicit binding with `#[binding(n)]`").with_span(&field.ty));
        } else {
            bindable_fields.push(field);
        }
    }

    if !errors.is_empty() {
        return Err(darling::Error::multiple(errors));
    }

    // ... and check if they are actually bindable.
    let assertions = bindable_fields.iter().map(|f| {
        let ty = &f.ty;
        let field_name = f.ident.as_ref().ok_or_else(|| darling::Error::custom("Can only bind a named field"))?;
        let assert_fn_name = format_ident!("{}_must_impl_BindableField", field_name);

        Ok(quote! {
            #[allow(non_snake_case)]
            const _: fn() = || {
                fn #assert_fn_name<T: #crate_path::prelude::BindableField>() {}
                #assert_fn_name::<#ty>();
            };
        })
    }).collect::<Result<Vec<_>, darling::Error>>()?;
    
    let layout_entries = bindable_fields.iter().map(|f| {
        let ty = &f.ty;
        let binding = f.get_binding()?;
        let binding = binding.as_ref().ok_or_else(|| darling::Error::custom("Bindable field must have explicit binding"))?;
        let visibility = match &f.get_visibility()? {
            Some(v) => v.clone(),
            None => {
                if misses_default_visibility {
                    return Err(darling::Error::custom(
                        "Bindable field had no explicit visibility declaration on a BindableStruct with no explicit visibility declaration"
                    ).with_span(&f.ty));
                }

                default_visibility.clone()
            }
        };

        Ok(quote! {
            <#ty as #crate_path::prelude::BindableField>::layout_entry(#binding, #visibility)
        })
    }).collect::<Result<Vec<_>, darling::Error>>()?;

    let bind_entries = bindable_fields.iter().map(|f| {
        let ident = f.ident.as_ref().ok_or_else(|| darling::Error::custom("Unnamed Bindable fields are not supported"))?;
        let binding = f.get_binding()?;
        let binding = binding.as_ref().ok_or_else(|| darling::Error::custom("Bindable field must have explicit binding"))?;

        Ok(quote! {
            #crate_path::prelude::BindableField::bind_group_entry(&self.#ident, #binding)
        })
    }).collect::<Result<Vec<_>, darling::Error>>()?;

    let impl_block = quote! {
        impl #impl_generics #crate_path::prelude::BindableStruct for #name #ty_generics #where_clause {
            fn bind_group_layout(device: &#wgpu_path::Device) -> #wgpu_path::BindGroupLayout {
                device.create_bind_group_layout(&#wgpu_path::BindGroupLayoutDescriptor {
                    label: Some(&format!("{} BGL", #name_str)),
                    entries: &[#(#layout_entries),*],
                })
            }

            fn bind_group(&self, device: &#wgpu_path::Device) -> #wgpu_path::BindGroup {
                device.create_bind_group(&#wgpu_path::BindGroupDescriptor {
                    label: Some(&format!("{} Bind Group", #name_str)),
                    layout: &Self::bind_group_layout(device),
                    entries: &[#(#bind_entries),*],
                })
            }
        }
    };

    Ok(quote! {
        #(#assertions)*
        #impl_block
    })
}

#[derive(FromDeriveInput)]
#[darling(forward_attrs(visibility), supports(struct_named))]
struct StructOpts {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<darling::util::Ignored, FieldOpts>,
    attrs: Vec<syn::Attribute>,

    /*#[darling(default)]
    visibility: Option<syn::Expr>*/
}

impl StructOpts {
    pub fn get_visibility(&self) -> syn::Result<Option<syn::Expr>> {
        self.attrs.iter()
            .find(|a| a.path().is_ident("visibility"))
            .map(|a| a.parse_args::<syn::Expr>())
            .transpose()
    }
}

#[derive(FromField)]
#[darling(forward_attrs(visibility, binding))]
struct FieldOpts {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,

    /*#[darling(default)]
    binding: Option<syn::LitInt>,
    #[darling(default)]
    visibility: Option<syn::Expr>*/
}

impl FieldOpts {
    pub fn get_binding(&self) -> syn::Result<Option<syn::LitInt>> {
        self.attrs.iter()
            .find(|a| a.path().is_ident("binding"))
            .map(|a| a.parse_args::<syn::LitInt>())
            .transpose()
    }

    pub fn get_visibility(&self) -> syn::Result<Option<syn::Expr>> {
        self.attrs.iter()
            .find(|a| a.path().is_ident("visibility"))
            .map(|a| a.parse_args::<syn::Expr>())
            .transpose()
    }
}