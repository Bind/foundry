use super::TraceIdentifier;
use ethers::{
    abi::{Abi, Address},
    prelude::ArtifactId,
};
use std::{borrow::Cow, collections::BTreeMap};

/// The local trace identifier keeps track of addresses that are instances of local contracts.
pub struct LocalTraceIdentifier {
    local_contracts: BTreeMap<Vec<u8>, (String, Abi)>,
}

impl LocalTraceIdentifier {
    pub fn new(known_contracts: &BTreeMap<ArtifactId, (Abi, Vec<u8>)>) -> Self {
        Self {
            local_contracts: known_contracts
                .iter()
                .map(|(id, (abi, runtime_code))| {
                    (runtime_code.clone(), (id.name.clone(), abi.clone()))
                })
                .collect(),
        }
    }
}

impl TraceIdentifier for LocalTraceIdentifier {
    fn identify_address(
        &self,
        _: &Address,
        code: Option<&Vec<u8>>,
    ) -> (Option<String>, Option<String>, Option<Cow<Abi>>) {
        code.map_or((None, None, None), |code| {
            self.local_contracts
                .iter()
                .find(|(known_code, _)| diff_score(known_code, code) < 0.1)
                .map_or((None, None, None), |(_, (name, abi))| {
                    (Some(name.clone()), Some(name.clone()), Some(Cow::Borrowed(abi)))
                })
        })
    }
}

/// Very simple fuzzy matching of contract bytecode.
///
/// Will fail for small contracts that are essentially all immutable variables.
fn diff_score(a: &[u8], b: &[u8]) -> f64 {
    let cutoff_len = usize::min(a.len(), b.len());
    if cutoff_len == 0 {
        return 1.0
    }

    let a = &a[..cutoff_len];
    let b = &b[..cutoff_len];
    let mut diff_chars = 0;
    for i in 0..cutoff_len {
        if a[i] != b[i] {
            diff_chars += 1;
        }
    }
    diff_chars as f64 / cutoff_len as f64
}