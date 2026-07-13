// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

// errors/mod.rs

pub(crate) mod application;
pub(crate) mod data_link;
pub(crate) mod internet;
mod link_layer;
pub(crate) mod transport;

use application::ApplicationError;
use data_link::DataLinkError;
use internet::InternetError;
pub use link_layer::LinkLayerError;
use thiserror::Error;
use transport::TransportError;

use crate::LinkType;

/// Error returned by the top-level packet parsing APIs.
///
/// The legacy variants remain available so that introducing explicit link
/// types does not alter the behaviour of Ethernet parsing. New link decoders
/// can add link-aware variants without forcing downstream exhaustive matches
/// to change again.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ParseError {
    #[error("Unsupported link type: {0}")]
    UnsupportedLinkType(LinkType),

    #[error("Packet too short: {0} bytes")]
    PacketTooShort(u8),

    #[error("Invalid DataLink segment: {0}")]
    InvalidDataLink(#[from] DataLinkError),

    #[error("Invalid link layer: {0}")]
    InvalidLinkLayer(#[from] LinkLayerError),

    #[error("Invalid Internet segment: {0}")]
    InvalidInternet(#[from] InternetError),

    #[error("Transport layer error: {0}")]
    Transport(#[from] TransportError),

    #[error("Application layer error: {0}")]
    Application(#[from] ApplicationError),
}

/// Backward-compatible name for [`ParseError`].
pub type ParsedPacketError = ParseError;
