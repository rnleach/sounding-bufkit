#![allow(missing_docs, unused_doc_comment)]
//! Error types for the sounding crate.
use std::num;

error_chain!{
    foreign_links {
        // Links to std errors
        Io(::std::io::Error);

        // Links to num crate parse errors
        ParseInt(num::ParseIntError);
        ParseFloat(num::ParseFloatError);
    }
}
