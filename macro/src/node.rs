use proc_macro2::TokenStream;
use syn::*;
use quote::{quote, ToTokens};
use convert_case::{Case, Casing};

pub fn expand(
    state: ItemStruct,
) -> Result<TokenStream> {    
    let ext = quote!{ frand_node::ext };
    let vis = &state.vis;
    let state_name = state.ident.clone();

    let state_snake_name = Ident::new(
        &state_name.to_string().to_case(Case::Snake), 
        state_name.span(),
    );

    let impl_params = state.generics.params.clone();
    let ty_params = impl_params.iter().map(|param|
        match param {
            GenericParam::Type(ty) => ty.ident.to_token_stream(),
            GenericParam::Lifetime(lifetime) => lifetime.lifetime.to_token_stream(),
            GenericParam::Const(const_param) => const_param.ident.to_token_stream(),
        }
    );
    let ty_params = quote! {
        #(#ty_params,)*
    };

    let impl_generics = {
        let lt_token = state.generics.lt_token.clone();
        let gt_token = state.generics.gt_token.clone();
        quote! {
            #lt_token #impl_params #gt_token
        }
    };

    let ty_generics = {
        let lt_token = state.generics.lt_token.clone();
        let gt_token = state.generics.gt_token.clone();
        quote! {
            #lt_token #ty_params #gt_token
        }
    };

    let fields: Vec<&Field> = match &state.fields {
        Fields::Named(fields_named) => fields_named.named.iter().collect(),
        _ => unimplemented!("not supported"),
    };  

    let viss: Vec<_> = fields.iter().map(|field| &field.vis).collect();
    let names: Vec<_> = fields.iter().filter_map(|field| field.ident.as_ref()).collect();
    let tys: Vec<_> = fields.iter().map(|field| &field.ty).collect();

    let id_delta_names: Vec<_> = fields.iter().filter_map(|field| 
        field.ident.as_ref().map(|name| {
            Ident::new(
                &format!("{}IdDelta", name.to_string()).to_case(Case::UpperSnake), 
                name.span(),
            )
        })        
    ).collect();

    let id_delta_end_names: Vec<_> = fields.iter().filter_map(|field| 
        field.ident.as_ref().map(|name| {
            Ident::new(
                &format!("{}IdDeltaEnd", name.to_string()).to_case(Case::UpperSnake), 
                name.span(),
            )
        })        
    ).collect();

    let node_sizes: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #ext::State>::NODE_SIZE }
    ).collect();

    let id_deltas: Vec<_> = (0..fields.len()).into_iter()
    .map(|index| {
        let mut tokens = quote!{ 1 };

        for i in 0..index {
            let node_size = &node_sizes[i];
            quote!{
                + #node_size
            }.to_tokens(&mut tokens);
        }

        tokens
    }).collect();

    let pascal_names: Vec<_> = fields.iter().filter_map(|field| 
        field.ident.as_ref().map(|name| {
            Ident::new(
                &name.to_string().to_case(Case::Pascal), 
                name.span(),
            )
        })        
    ).collect();

    let message_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #ext::State>::Message }
    ).collect();

    let accesser_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #ext::State>::Accesser }
    ).collect();
    
    Ok(quote!{       
        impl #impl_generics #ext::State for #state_name #ty_generics {
            const NODE_SIZE: #ext::IdSize = 1 #(+ #node_sizes)*;
            const NODE_ALT_SIZE: #ext::AltSize = 0;
    
            type Message = #state_snake_name::Message #ty_generics;
            type Emitter = #state_snake_name::Emitter #ty_generics;
            type Accesser<CS: #ext::System> = #state_snake_name::Accesser<CS, #ty_params>;
            type Node<'n, CS: #ext::System> = #state_snake_name::Node<'n, CS, #ty_params>;

            fn from_payload(payload: &#ext::Payload) -> Self {
                #ext::Payload::to_state(payload)
            }

            fn to_payload(&self) -> #ext::Payload {
                #ext::Payload::from_state(self)
            }       
                    
            fn into_message(self) -> Self::Message {
                #state_snake_name::Message::State(self)
            }  
        }

        #vis mod #state_snake_name {
            use super::*;

            #(
                const #id_delta_names: #ext::IdDelta = #id_deltas;
                const #id_delta_end_names: #ext::IdDelta = #id_delta_names + #node_sizes;
            )*
            
            #[derive(Debug, Clone)]
            pub enum Message #impl_generics {
                #(#pascal_names(#[allow(dead_code)] #message_tys),)*
                State(#[allow(dead_code)] #state_name #ty_generics),
            }

            #[derive(Debug, Clone)]
            pub struct Emitter #impl_generics {
                callback: #ext::Callback<Message #ty_generics>,
                #(#viss #names: <#tys as #ext::State>::Emitter,)*
            }

            #[derive(Debug, Clone)]
            pub struct Accesser<CS: #ext::System, #impl_params> {
                access: #ext::RcAccess<#state_name #ty_generics, CS>,
                #(#viss #names: #accesser_tys<CS>,)*
            }

            #[derive(Debug)]
            pub struct Node<'n, CS: #ext::System, #impl_params> {
                emitter: &'n Emitter #ty_generics,
                accesser: &'n Accesser<CS, #ty_params>,
                consensus: &'n std::sync::Arc<std::sync::RwLockReadGuard<'n, CS>>,                
                alt: &'n #ext::Alt,      
                #(#viss #names: <#tys as #ext::State>::Node<'n, CS>,)*
            }
        
            impl #impl_generics #ext::Fallback for #state_name #ty_generics {
                fn fallback<CS: #ext::System>(
                    node: Node<'_, CS, #ty_params>, 
                    message: Message #ty_generics, 
                    delta: Option<std::time::Duration>,
                ) {
                    match message {
                        #(Message::#pascal_names(message) => <#tys>::handle(node.#names, message, delta),)*
                        Message::State(state) => {
                            #(<#tys>::handle(node.#names, state.#names.into_message(), delta);)*
                        },
                    } 
                }
            }

            impl #impl_generics #ext::Message for Message #ty_generics {    
                type State = #state_name #ty_generics;

                fn from_packet(
                    packet: &#ext::Packet,
                    parent_key: #ext::Key,
                    depth: usize,                
                ) -> #ext::Result<Self> {
                    Ok(match packet.key().consist().id() - parent_key.consist().id() {
                        0 => Ok(Self::State(
                            #ext::State::from_payload(packet.payload())
                        )),
                        #(#id_delta_names..#id_delta_end_names => Ok(
                            Message::#pascal_names(#message_tys::from_packet(
                                packet, 
                                #ext::Key::new(
                                    parent_key.consist().access(#id_delta_names, <#state_name #ty_generics>::NODE_ALT_SIZE),
                                    parent_key.alt(),
                                ), 
                                depth + 1,
                            )?)
                        ),)*
                        id_delta => Err(#ext::PacketError::new(
                            packet.clone(),
                            Some(id_delta),
                            Some(depth),
                            format!(
                                "{}: unknown id_delta", 
                                std::any::type_name::<Self>(), 
                            ),
                        )),
                    }?)  
                }     

                fn to_packet(
                    &self, 
                    key: #ext::Key,
                ) -> #ext::Packet {     
                    match self {
                        #(Self::#pascal_names(message) => message.to_packet(key),)*
                        Self::State(state) => #ext::Packet::new(
                            key, 
                            #ext::State::to_payload(state),
                        ),
                    }
                }        

                fn apply_to(&self, state: &mut #state_name #ty_generics) {
                    match self {
                        #(Self::#pascal_names(#names) => #names.apply_to(&mut state.#names),)*
                        Self::State(new_state) => *state = new_state.clone(),
                    }
                }  
            } 

            impl #impl_generics #ext::Emitter<#state_name #ty_generics> for Emitter #ty_generics {  
                fn callback(&self) -> &#ext::Callback<Message #ty_generics> { &self.callback }

                fn new(
                    callback: #ext::Callback<Message #ty_generics>,
                ) -> Self {
                    Self { 
                        #(
                            #names: #ext::Emitter::new(
                                #ext::Callback::access(
                                    callback.clone(), 
                                    #id_delta_names, <#state_name #ty_generics>::NODE_ALT_SIZE,
                                    |_, message| Message::#pascal_names(message),
                                ),
                            ),
                        )*
                        callback, 
                    }
                }
            }

            impl<CS: System, #impl_params> std::ops::Deref for Accesser<CS, #ty_params> {
                type Target = #ext::RcAccess<#state_name #ty_generics, CS>;
                fn deref(&self) -> &Self::Target { &self.access }
            }
        
            impl<CS: System, #impl_params> #ext::Accesser<#state_name #ty_generics, CS> for Accesser<CS, #ty_params> {
                fn new(
                    access: #ext::RcAccess<#state_name #ty_generics, CS>,
                ) -> Self {
                    Self { 
                        #(#names: #ext::Accesser::new(
                            #ext::RcAccess::access(
                                access.clone(), 
                                #id_delta_names, <#state_name #ty_generics>::NODE_ALT_SIZE,
                                |state, _| &state.#names,
                            ),
                        ),)*
                        access, 
                    }
                }
            }

            impl<'n, CS: #ext::System, #impl_params> std::ops::Deref for Node<'n, CS, #ty_params> {
                type Target = #state_name #ty_generics;
                fn deref(&self) -> &Self::Target { 
                    (self.accesser.access)(self.consensus, *self.alt)
                }
            }

            impl<'n, CS: #ext::System, #impl_params> #ext::Node<'n, #state_name #ty_generics> for Node<'n, CS, #ty_params> {
                fn alt(&self) -> &#ext::Alt { self.alt }
                fn emitter(&self) -> &Emitter #ty_generics { self.emitter }
            }

            impl<'n, CS: #ext::System, #impl_params> #ext::NewNode<'n, #state_name #ty_generics, CS> for Node<'n, CS, #ty_params> {
                fn new(
                    emitter: &'n Emitter #ty_generics,
                    accesser: &'n Accesser<CS, #ty_params>,
                    consensus: &'n std::sync::Arc<std::sync::RwLockReadGuard<'n, CS>>,
                    alt: &'n #ext::Alt,     
                ) -> Self {
                    Self { 
                        emitter, 
                        accesser,
                        #(#names: #ext::NewNode::new(
                            &emitter.#names, 
                            &accesser.#names, 
                            consensus, 
                            alt, 
                        ),)*
                        consensus, 
                        alt,
                    }
                }
        
                fn new_alt(
                    &self,
                    alt: #ext::Alt,         
                ) -> #ext::ConsensusRead<'n, #state_name #ty_generics, CS> {
                    #ext::ConsensusRead::new(
                        self.emitter, 
                        self.accesser, 
                        self.consensus.clone(), 
                        alt,
                    )
                }
            }
        }
    })
}