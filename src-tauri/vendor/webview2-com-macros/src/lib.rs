use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, Result, Token, TypePath, Visibility,
};

struct CallbackTypes {
    pub interface: TypePath,
    pub arg_1: TypePath,
    pub arg_2: Option<TypePath>,
}

impl Parse for CallbackTypes {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);
        let args = content.parse_terminated(TypePath::parse, Token![,])?;
        input.parse::<Token![;]>()?;
        let mut args = args.into_iter();
        if let (Some(interface), Some(arg_1), arg_2) = (args.next(), args.next(), args.next()) {
            Ok(CallbackTypes {
                interface,
                arg_1,
                arg_2,
            })
        } else {
            Err(content.error("expected (interface, arg_1, arg_2)"))
        }
    }
}

struct CallbackStruct {
    pub vis: Visibility,
    _struct_token: Token![struct],
    pub ident: Ident,
    pub args: CallbackTypes,
}

impl Parse for CallbackStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(CallbackStruct {
            vis: input.parse()?,
            _struct_token: input.parse()?,
            ident: input.parse()?,
            args: input.parse()?,
        })
    }
}

/// Implement a `CompletedCallback` using the types specified as tuple struct fields.
#[proc_macro_attribute]
pub fn completed_callback(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as CallbackStruct);
    impl_completed_callback(&ast)
}

fn impl_completed_callback(ast: &CallbackStruct) -> TokenStream {
    let vis = &ast.vis;

    let name = &ast.ident;
    let name_impl = format_ident!("{name}_Impl");
    let closure = get_closure(name);
    let interface = &ast.args.interface;
    let interface = interface
        .path
        .segments
        .last()
        .expect("completed_callback should always specify an interface");
    let interface_impl = format_ident!("{}_Impl", interface.ident);

    let arg_1 = &ast.args.arg_1;
    let arg_2 = &ast.args.arg_2;

    let msg = format!("Implementation of [`{interface_impl}`].");

    let gen = match arg_2 {
        Some(arg_2) => quote! {
            type #closure = CompletedClosure<#arg_1, #arg_2>;

            #[doc = #msg]
            #[implement(#interface)]
            #vis struct #name(::std::cell::UnsafeCell<Option<#closure>>);

            impl #name {
                pub fn create(
                    closure: #closure,
                ) -> #interface {
                    Self(Some(closure).into()).into()
                }

                pub fn wait_for_async_operation(
                    closure: Box<
                        dyn FnOnce(#interface) -> crate::Result<()>,
                    >,
                    completed: #closure,
                ) -> crate::Result<()> {
                    let (tx, rx) = ::std::sync::mpsc::channel();
                    let completed: #closure =
                        Box::new(move |arg_1, arg_2| -> ::windows::core::Result<()> {
                            let result = completed(arg_1, arg_2).map_err(crate::Error::WindowsError);
                            tx.send(result).expect("send over mpsc channel");
                            Ok(())
                        });
                    let callback = Self::create(completed);

                    closure(callback)?;
                    crate::wait_with_pump(rx)?
                }
            }

            #[allow(non_snake_case)]
            impl #interface_impl for #name_impl {
                fn Invoke<'a>(
                    &self,
                    arg_1: <#arg_1 as InvokeArg<'a>>::Input,
                    arg_2: <#arg_2 as InvokeArg<'a>>::Input,
                ) -> ::windows::core::Result<()> {
                    match unsafe { (*self.0.get()).take() } {
                        Some(completed) => completed(
                            <#arg_1 as InvokeArg<'a>>::convert(arg_1),
                            <#arg_2 as InvokeArg<'a>>::convert(arg_2),
                        ),
                        None => Ok(()),
                    }
                }
            }
        },
        None => quote! {
            type #closure = Box<dyn FnOnce(<#arg_1 as ClosureArg>::Output) -> ::windows::core::Result<()>>;

            #[doc = #msg]
            #[implement(#interface)]
            #vis struct #name(::std::cell::UnsafeCell<Option<#closure>>);

            impl #name {
                pub fn create(
                    closure: #closure,
                ) -> #interface {
                    Self(Some(closure).into()).into()
                }

                pub fn wait_for_async_operation(
                    closure: Box<
                        dyn FnOnce(#interface) -> crate::Result<()>,
                    >,
                    completed: #closure,
                ) -> crate::Result<()> {
                    let (tx, rx) = ::std::sync::mpsc::channel();
                    let completed: #closure =
                        Box::new(move |arg_1| -> ::windows::core::Result<()> {
                            let result = completed(arg_1).map_err(crate::Error::WindowsError);
                            tx.send(result).expect("send over mpsc channel");
                            Ok(())
                        });
                    let callback = Self::create(completed);

                    closure(callback)?;
                    crate::wait_with_pump(rx)?
                }
            }

            #[allow(non_snake_case)]
            impl #interface_impl for #name_impl {
                fn Invoke<'a>(
                    &self,
                    arg_1: <#arg_1 as InvokeArg<'a>>::Input,
                ) -> ::windows::core::Result<()> {
                    match unsafe { (*self.0.get()).take() } {
                        Some(completed) => completed(
                            <#arg_1 as InvokeArg<'a>>::convert(arg_1),
                        ),
                        None => Ok(()),
                    }
                }
            }
        },
    };

    gen.into()
}

/// Implement an `EventCallback` using the types specified as tuple struct fields.
#[proc_macro_attribute]
pub fn event_callback(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as CallbackStruct);
    impl_event_callback(&ast)
}

fn impl_event_callback(ast: &CallbackStruct) -> TokenStream {
    let vis = &ast.vis;

    let name = &ast.ident;
    let name_impl = format_ident!("{name}_Impl");
    let closure = get_closure(name);

    let interface = &ast.args.interface;
    let interface = interface
        .path
        .segments
        .last()
        .expect("event_callback should always specify an interface");
    let interface_impl = format_ident!("{}_Impl", interface.ident);

    let arg_1 = &ast.args.arg_1;
    let arg_2 = &ast
        .args
        .arg_2
        .as_ref()
        .expect("event_callback should always have 2 arguments");

    let msg = format!("Implementation of [`{interface_impl}`].");

    let gen = quote! {
        type #closure = EventClosure<#arg_1, #arg_2>;

        #[doc = #msg]
        #[implement(#interface)]
        #vis struct #name(::std::cell::UnsafeCell<#closure>);

        impl #name {
            pub fn create(
                closure: #closure,
            ) -> #interface {
                Self(closure.into()).into()
            }
        }

        #[allow(non_snake_case)]
        impl #interface_impl for  #name_impl {
            fn Invoke<'a>(
                &self,
                arg_1: <#arg_1 as InvokeArg<'a>>::Input,
                arg_2: <#arg_2 as InvokeArg<'a>>::Input,
            ) -> ::windows::core::Result<()> {
                unsafe { (*self.0.get())(
                    <#arg_1 as InvokeArg<'a>>::convert(arg_1),
                    <#arg_2 as InvokeArg<'a>>::convert(arg_2),
                ) }
            }
        }
    };

    gen.into()
}

fn get_closure(name: &Ident) -> Ident {
    format_ident!("{name}Closure")
}
