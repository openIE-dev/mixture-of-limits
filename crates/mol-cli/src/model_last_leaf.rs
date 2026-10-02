//! Bridge: mol-adapters ModelLastPort → mol-cascade ModelStub.

use mol_adapters::ModelLastPort;
use mol_cascade::ModelStub;
use mol_core::{MolRequest, Result};

/// Model LAST leaf that delegates to a [`ModelLastPort`] (stub or endpoint).
pub struct ModelLastLeaf {
    inner: Box<dyn ModelLastPort>,
}

impl ModelLastLeaf {
    /// Wrap a Model LAST port.
    pub fn new(inner: Box<dyn ModelLastPort>) -> Self {
        Self { inner }
    }
}

impl ModelStub for ModelLastLeaf {
    fn generate(&self, req: &MolRequest) -> Result<String> {
        let p = self.inner.propose(req)?;
        // Honesty fence: never copy invented measured_j into cascade answer path.
        debug_assert!(p.measured_j.is_none());
        Ok(p.text)
    }
}
