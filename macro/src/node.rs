use proc_macro2::TokenStream;
use syn::*;
use quote::quote;
use convert_case::{Case, Casing};

pub type NodeId = u32;

pub fn expand(
    state: &ItemStruct,
) -> Result<TokenStream> {    
    let mp = quote!{ frand_node::macro_prelude };

    let vis = &state.vis;
    let state_name = state.ident.clone();

    let message_name = Ident::new(
        &format!("{}Message", state_name.to_string()).to_case(Case::Pascal), 
        state_name.span(),
    );

    let consensus_name = Ident::new(
        &format!("{}Consensus", state_name.to_string()).to_case(Case::Pascal), 
        state_name.span(),
    );

    let node_name = Ident::new(
        &format!("{}Node", state_name.to_string()).to_case(Case::Pascal), 
        state_name.span(),
    );

    let fields: Vec<&Field> = match &state.fields {
        Fields::Named(fields_named) => fields_named.named.iter().collect(),
        _ => unimplemented!("not supported"),
    };  

    let viss: Vec<_> = fields.iter().map(|field| &field.vis).collect();
    let indexes: Vec<_> = (0..fields.len() as NodeId).into_iter().collect();
    let names: Vec<_> = fields.iter().filter_map(|field| field.ident.as_ref()).collect();
    let tys: Vec<_> = fields.iter().map(|field| &field.ty).collect();

    let message_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #mp::State>::Message }
    ).collect();

    let consensus_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #mp::State>::Consensus }
    ).collect();

    let node_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #mp::State>::Node }
    ).collect();

    Ok(quote!{
        #[derive(Debug, Clone)]
        #vis enum #message_name {
            #(#[allow(non_camel_case_types)] #names(#[allow(dead_code)] #message_tys),)*
            #[allow(non_camel_case_types)] State(#[allow(dead_code)] #state_name),
        }

        #[derive(Debug, Clone)]
        #vis struct #consensus_name {
            key: #mp::NodeKey,
            #(#viss #names: #consensus_tys,)*
        }

        #[derive(Debug, Clone)]
        #vis struct #node_name {
            key: #mp::NodeKey,
            reporter: #mp::Reporter,
            #(#viss #names: #node_tys,)*
        }

        impl #mp::State for #state_name {
            type Message = #message_name;
            type Consensus = #consensus_name;
            type Node = #node_name;

            fn apply(
                &mut self, 
                depth: usize, 
                packet: #mp::Packet,
            ) -> core::result::Result<(), #mp::PacketError> {
                match packet.get_id(depth) {
                    #(Some(#indexes) => self.#names.apply(depth+1, packet),)*
                    Some(_) => Err(packet.error(depth, "unknown id")),
                    None => Ok(*self = packet.read_state()),
                }
            }    

            fn apply_message(
                &mut self,  
                message: Self::Message,
            ) {
                match message {
                    #(Self::Message::#names(message) => self.#names.apply_message(message),)*
                    Self::Message::State(state) => *self = state,
                }
            }
        }

        impl #mp::Message for #message_name {
            fn from_packet(depth: usize, packet: &#mp::Packet) -> core::result::Result<Self, #mp::PacketError> {
                Ok(match packet.get_id(depth) {
                    #(Some(#indexes) => Ok(
                        #message_name::#names(#message_tys::from_packet(depth + 1, packet)?)
                    ),)*
                    Some(_) => Err(packet.error(depth, "unknown id")),
                    None => Ok(Self::State(packet.read_state())),
                }?)     
            }
        }

        impl Default for #consensus_name {      
            fn default() -> Self { Self::new(vec![], None) }
        }

        impl #mp::Consensus<#state_name> for #consensus_name {  
            fn new(
                mut key: Vec<#mp::NodeId>,
                id: Option<#mp::NodeId>,
            ) -> Self {
                if let Some(id) = id { key.push(id); }
                
                Self { 
                    key: key.clone().into_boxed_slice(),   
                    #(#names: #mp::Consensus::new(key.clone(), Some(#indexes)),)*                     
                }
            }
    
            fn new_node(&self, reporter: &#mp::Reporter) -> #node_name {
                #node_name::new_from(self, reporter)
            }
            
            fn clone_state(&self) -> #state_name { 
                #state_name {
                    #(#names: self.#names.clone_state(),)*   
                }
            }

            fn apply(
                &mut self, 
                depth: usize, 
                packet: &#mp::Packet,
            ) -> core::result::Result<(), #mp::PacketError> {
                match packet.get_id(depth) {
                    #(Some(#indexes) => self.#names.apply(depth+1, packet),)*
                    Some(_) => Err(packet.error(depth, "unknown id")),
                    None => Ok(self.apply_state(packet.read_state())),
                } 
            }

            fn apply_state(&mut self, state: #state_name) {
                #(self.#names.apply_state(state.#names);)*       
            }
            
            fn apply_export(
                &mut self, 
                depth: usize, 
                packet: &#mp::Packet,
            ) -> core::result::Result<#message_name, #mp::PacketError> {
                match packet.get_id(depth) {
                    #(Some(#indexes) => {
                        Ok(#message_name::#names(self.#names.apply_export(depth+1, packet)?))
                    },)*
                    Some(_) => Err(packet.error(depth, "unknown id")),
                    None => {
                        let state: #state_name = packet.read_state();    
                        self.apply_state(state.clone());       
                        Ok(#message_name::State(state))
                    },
                }        
            }
        }

        impl #mp::Node<#state_name> for #node_name { 
            type State = #state_name;

            fn new_from(
                consensus: &#consensus_name,
                reporter: &#mp::Reporter,
            ) -> Self {
                Self { 
                    key: consensus.key.clone(),
                    reporter: reporter.clone(),
                    #(#names: #mp::Node::new_from(&consensus.#names, reporter),)*  
                }
            }

            fn clone_state(&self) -> #state_name { 
                #state_name {
                    #(#names: self.#names.clone_state(),)*   
                }
            }
        }

        impl AsRef<Self> for #node_name {
            #[inline] fn as_ref(&self) -> &Self { self }
        }

        impl #mp::Emitter<#state_name> for #node_name {  
            fn emit(&self, state: #state_name) {
                self.reporter.report(&self.key, state)
            }

            fn emit_future<Fu>(&self, future: Fu) 
            where Fu: 'static + std::future::Future<Output = #state_name> + Send {
                self.reporter.report_future(&self.key, future)
            }
        }
    })
}