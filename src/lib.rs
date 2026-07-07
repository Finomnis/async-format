#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
// That's just a bad lint, in many cases I want two ifs for readability
#![allow(clippy::collapsible_if)]
