use proc_macro2::TokenStream;
use syn::*;
use quote::{quote, ToTokens};
use convert_case::{Case, Casing};

pub fn expand(
    mut state: ItemStruct,
) -> Result<TokenStream> {    
    let mp = quote!{ frand_node::macro_prelude };

    let vis = &state.vis;
    let node_name = state.ident.clone();

    let state_name = Ident::new(
        &format!("{}State", node_name.to_string()).to_case(Case::Pascal), 
        node_name.span(),
    );

    state.ident = state_name.clone();

    let message_name = Ident::new(
        &format!("{}Message", node_name.to_string()).to_case(Case::Pascal), 
        node_name.span(),
    );

    let node_index_starts_name = Ident::new(
        &format!("{}IndexStarts", node_name.to_string()).to_case(Case::Snake), 
        node_name.span(),
    );

    let node_index_ends_name = Ident::new(
        &format!("{}IndexEnds", node_name.to_string()).to_case(Case::Snake), 
        node_name.span(),
    );

    let fields: Vec<&Field> = match &state.fields {
        Fields::Named(fields_named) => fields_named.named.iter().collect(),
        _ => unimplemented!("not supported"),
    };  

    let viss: Vec<_> = fields.iter().map(|field| &field.vis).collect();
    let names: Vec<_> = fields.iter().filter_map(|field| field.ident.as_ref()).collect();
    let tys: Vec<_> = fields.iter().map(|field| &field.ty).collect();

    let pascal_names: Vec<_> = fields.iter().filter_map(|field| 
        field.ident.as_ref().map(|name| {
            Ident::new(
                &name.to_string().to_case(Case::Pascal), 
                name.span(),
            )
        })        
    ).collect();

    let upper_snake_names: Vec<_> = fields.iter().filter_map(|field| 
        field.ident.as_ref().map(|name| {
            Ident::new(
                &name.to_string().to_case(Case::UpperSnake), 
                name.span(),
            )
        })        
    ).collect();

    let message_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #mp::Accessor>::Message }
    ).collect();

    let node_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #mp::Accessor>::Node }
    ).collect();

    let node_sizes: Vec<_> = tys.iter().map(|ty| 
        quote!{ <<#ty as #mp::Accessor>::State as #mp::State>::NODE_SIZE }
    ).collect();

    let node_indexes: Vec<_> = (0..fields.len()).into_iter()
    .map(|index| {
        let mut tokens = quote!{1};

        for i in 0..index {
            let node_size = &node_sizes[i];
            quote!{
                + #node_size
            }.to_tokens(&mut tokens);
        }

        tokens
    }).collect(); 

    let state_attrs = state.attrs;
    let state_generics = state.generics;
    let state_fields: Vec<_> = fields.iter().map(|field| {
        let attrs = &field.attrs;
        let vis = &field.vis;
        let ident = &field.ident;
        let ty = &field.ty;
        quote! { #(#attrs)* #vis #ident: <#ty as #mp::Accessor>::State }
    }).collect();

    Ok(quote!{        
        #(#state_attrs)*
        #vis struct #state_name #state_generics {
            #(#state_fields,)*
        }

        #[derive(Debug, Clone)]
        #vis enum #message_name {
            #(#[allow(non_camel_case_types)] #pascal_names(#[allow(dead_code)] #message_tys),)*
            #[allow(non_camel_case_types)] State(#[allow(dead_code)] #state_name),
        }

        #[derive(Debug, Clone)]
        #vis struct #node_name {
            key: #mp::Key,
            emitter: Option<#mp::Emitter>,
            #(#viss #names: #node_tys,)*
        }

        mod #node_index_starts_name {
            #[allow(unused_imports)] use super::*;
            #(pub const #upper_snake_names: #mp::Index = #node_indexes;)*
        }

        mod #node_index_ends_name {
            #[allow(unused_imports)] use super::*;
            #(pub const #upper_snake_names: #mp::Index = #node_indexes + #node_sizes;)*
        }

        impl #mp::Accessor for #state_name  {
            type State = Self;
            type Message = #message_name;
            type Node = #node_name;
        }

        impl #mp::Emitable for #state_name {}

        impl #mp::State for #state_name {
            const NODE_SIZE: #mp::Index = 1 #(+ #node_sizes)*;

            fn apply(
                &mut self,  
                message: Self::Message,
            ) {
                match message {
                    #(Self::Message::#pascal_names(message) => self.#names.apply(message),)*
                    Self::Message::State(state) => *self = state,
                }
            }
        }

        impl #mp::Message for #message_name {
            fn from_packet_message(
                parent_key: #mp::Key,
                packet: &#mp::PacketMessage, 
            ) -> core::result::Result<Self, #mp::MessageError> {      
                match packet.key() - parent_key {
                    0 => Ok(Self::State(unsafe { 
                        #mp::State::from_emitable(packet.payload()) 
                    })),
                    #(#node_index_starts_name::#upper_snake_names..#node_index_ends_name::#upper_snake_names => Ok(#message_name::#pascal_names(
                        #message_tys::from_packet_message(
                            parent_key + #node_index_starts_name::#upper_snake_names, 
                            packet,
                        )?
                    )),)*
                    index => Err(#mp::MessageError::new(
                        packet.key(),
                        Some(index),
                        format!("{}: unknown index", std::any::type_name::<Self>()),
                    )),
                }    
            } 

            fn from_packet(
                parent_key: #mp::Key,
                packet: &#mp::Packet, 
            ) -> core::result::Result<Self, #mp::PacketError> {
                Ok(match packet.key() - parent_key {
                    0 => Ok(Self::State(
                        packet.read_state()
                    )),
                    #(#node_index_starts_name::#upper_snake_names..#node_index_ends_name::#upper_snake_names => Ok(
                        #message_name::#pascal_names(#message_tys::from_packet(
                            parent_key + #node_index_starts_name::#upper_snake_names, 
                            packet, 
                        )?)
                    ),)*
                    index => Err(#mp::PacketError::new(
                        packet.clone(),
                        Some(index),
                        format!("{}: unknown index", std::any::type_name::<Self>()),
                    )),
                }?)     
            }

            fn to_packet(
                &self,
                key: #mp::Key, 
            ) -> core::result::Result<#mp::Packet, #mp::MessageError> {       
                match self {
                    #(Self::#pascal_names(message) => message.to_packet(key),)*
                    Self::State(state) => Ok(#mp::Packet::new(key, state)),
                }
            }
        }

        impl Default for #node_name { 
            fn default() -> Self { Self::new(#mp::Key::default(), 0, None) }
        }

        impl #mp::Accessor for #node_name  {
            type State = #state_name;
            type Message = #message_name;
            type Node = #node_name;
        }

        impl #mp::Fallback for #node_name {
            fn fallback(&self, message: Self::Message, delta: Option<f32>) {
                match message {
                    #(#message_name::#pascal_names(message) => self.#names.handle(message, delta),)*
                    #message_name::State(_) => (),
                } 
            }
        }
        
        impl #mp::Node<#state_name> for #node_name { 
            fn key(&self) -> #mp::Key { self.key }
            fn emitter(&self) -> Option<&#mp::Emitter> { self.emitter.as_ref() }

            fn clone_state(&self) -> #state_name { 
                #state_name {
                    #(#names: self.#names.clone_state(),)*   
                }
            }
        }
        
        impl #mp::NewNode<#state_name> for #node_name { 
            fn new(
                mut key: #mp::Key,
                index: #mp::Index,
                emitter: Option<#mp::Emitter>,
            ) -> Self {
                key = key + index;

                Self { 
                    key,   
                    emitter: emitter.clone(),
                    #(#names: #mp::NewNode::new(
                        key, #node_index_starts_name::#upper_snake_names, emitter.clone(),
                    ),)*
                }
            }
        }
        
        impl #mp::Consensus<#state_name> for #node_name { 
            fn new_from(
                node: &Self,
                emitter: Option<#mp::Emitter>,
            ) -> Self {
                Self {
                    key: node.key,
                    emitter: emitter.clone(),
                    #(#names: #mp::Consensus::new_from(&node.#names, emitter.clone()),)*
                }
            }

            fn set_emitter(&mut self, emitter: Option<#mp::Emitter>) { 
                self.emitter = emitter.clone(); 
                #(self.#names.set_emitter(emitter.clone());)*   
            }

            fn apply(&self, message: #message_name) {
                match message {
                    #(#message_name::#pascal_names(#names) => self.#names.apply(#names),)*
                    #message_name::State(state) => {
                        self.apply_state(state.clone());
                        #(self.#names.emit(state.#names);)*                    
                    },
                } 
            }

            fn apply_state(&self, state: #state_name) {
                #(self.#names.apply_state(state.#names);)*       
            }
        }
    })
}