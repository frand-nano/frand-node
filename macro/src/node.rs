use proc_macro2::TokenStream;
use syn::*;
use quote::quote;
use convert_case::{Case, Casing};

pub type AnchorId = u32;

pub fn expand(
    state: &ItemStruct,
) -> Result<TokenStream> {    
    let mp = quote!{ frand_node::macro_prelude };

    let vis = &state.vis;
    let state_name = state.ident.clone();

    let anchor_name = Ident::new(
        &format!("{}Anchor", state.ident.to_string()).to_case(Case::Pascal), 
        state.ident.span(),
    );

    let message_name = Ident::new(
        &format!("{}Message", state_name.to_string()).to_case(Case::Pascal), 
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
    let indexes: Vec<_> = (0..fields.len() as AnchorId).into_iter().collect();
    let names: Vec<_> = fields.iter().filter_map(|field| field.ident.as_ref()).collect();
    let tys: Vec<_> = fields.iter().map(|field| &field.ty).collect();

    let anchor_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #mp::State>::Anchor }
    ).collect();

    let message_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #mp::State>::Message }
    ).collect();

    let node_tys: Vec<_> = tys.iter().map(|ty| 
        quote!{ <#ty as #mp::State>::Node }
    ).collect();

    Ok(quote!{
        #[derive(Debug, Clone)]
        #vis struct #anchor_name {
            key: #mp::AnchorKey,
            reporter: #mp::Reporter,
            #(#viss #names: #anchor_tys,)*
        }

        #[derive(Debug, Clone)]
        #vis enum #message_name {
            #(#[allow(non_camel_case_types)] #names(#[allow(dead_code)] #message_tys),)*
            #[allow(non_camel_case_types)] State(#[allow(dead_code)] #state_name),
        }

        #vis struct #node_name<'sn> {
            anchor: &'sn #anchor_name,
            #(#viss #names: #node_tys<'sn>,)*
        }

        impl #mp::State for #state_name {
            type Anchor = #anchor_name;
            type Message = #message_name;
            type Node<'sn> = #node_name<'sn>;

            fn apply(
                &mut self,  
                depth: usize,
                packet: &#mp::Packet,
            ) -> #mp::anyhow::Result<()> {
                match packet.get_id(depth) {
                    #(Some(#indexes) => self.#names.apply(depth+1, packet),)*
                    Some(_) => Err(packet.error(depth, "unknown id")),
                    None => Ok(*self = packet.read_state()),
                }
            }
        }

        impl #mp::Anchor for #anchor_name {
            fn key(&self) -> &#mp::AnchorKey { &self.key }
            fn reporter(&self) -> &#mp::Reporter { &self.reporter }
        
            fn new(
                mut key: Vec<#mp::AnchorId>,
                id: Option<#mp::AnchorId>,
                reporter: &#mp::Reporter,
            ) -> Self {
                if let Some(id) = id { key.push(id); }
        
                Self { 
                    #(#names: #anchor_tys::new(key.clone(), Some(#indexes), reporter),)*   
                    key: key.into_boxed_slice(),      
                    reporter: reporter.clone(),
                }
            }
        }

        impl Default for #anchor_name {
            fn default() -> Self { Self::new(vec![], None, &(|_|()).into()) }
        }

        impl #mp::Emitter<#state_name> for #anchor_name {
            fn emit(&self, state: #state_name) {
                self.reporter().report(state.into_packet(self.key().clone()))
            }
        }

        impl #mp::Message for #message_name {
            fn from_packet(depth: usize, packet: &#mp::Packet) -> #mp::anyhow::Result<Self> {
                Ok(match packet.get_id(depth) {
                    #(Some(#indexes) => Ok(
                        #message_name::#names(#message_tys::from_packet(depth + 1, packet)?)
                    ),)*
                    Some(_) => Err(packet.error(depth, "unknown id")),
                    None => Ok(Self::State(packet.read_state())),
                }?)     
            }
        }
                
        impl #mp::Emitter<#state_name> for #node_name<'_> {
            fn emit(&self, state: #state_name) { self.anchor.emit(state) }
        }

        impl<'sn> #mp::Node<'sn, #state_name> for #node_name<'sn> {
            fn new(state: &'sn #state_name, anchor: &'sn #anchor_name) -> Self { 
                Self { 
                    anchor, 
                    #(#names: #node_tys::new(&state.#names, &anchor.#names),)*   
                } 
            } 

            fn clone_state(&self) -> #state_name { 
                #state_name {
                    #(#names: self.#names.clone_state(),)*   
                }
            }
            
            fn apply_export(
                &mut self, 
                depth: usize, 
                packet: &#mp::Packet,
            ) -> #mp::anyhow::Result<#message_name> {
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

        impl #node_name<'_> {
            pub fn apply_state(&mut self, state: #state_name) {
                #(self.#names.apply_state(state.#names);)*       
            }
        }
    })
}