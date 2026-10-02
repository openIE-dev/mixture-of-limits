//! Type-enforced replayability classes.
//!
//! `ModelGenerated` **cannot** coerce to `Deterministic`. Coercion is only allowed
//! along the weakening path: Deterministic → RetrievedCited → Composed → ModelGenerated.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;

use crate::error::{MolError, Result};

/// Replayability class (wire / receipt tag).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayClass {
    /// Bit-exact closed-form / LUT / proof — strongest.
    Deterministic,
    /// Retrieved with cite ids under authoritative structure.
    RetrievedCited,
    /// Deterministic composition of cited / closed-form parts.
    Composed,
    /// Stochastic / generative — weakest; never promotes to Deterministic.
    ModelGenerated,
}

impl ReplayClass {
    /// Strength rank (0 = strongest).
    pub const fn rank(self) -> u8 {
        match self {
            Self::Deterministic => 0,
            Self::RetrievedCited => 1,
            Self::Composed => 2,
            Self::ModelGenerated => 3,
        }
    }

    /// True if `self` may weaken into `target` (never strengthen).
    pub const fn can_weaken_to(self, target: Self) -> bool {
        self.rank() <= target.rank()
    }

    /// Wire label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Deterministic => "deterministic",
            Self::RetrievedCited => "retrieved_cited",
            Self::Composed => "composed",
            Self::ModelGenerated => "model_generated",
        }
    }
}

impl fmt::Display for ReplayClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

mod sealed {
    pub trait Sealed {}
}

/// Marker for type-level replay class.
pub trait ReplayMarker: sealed::Sealed + Copy + Default {
    /// Corresponding wire enum.
    const CLASS: ReplayClass;
}

/// Type-level Deterministic.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Deterministic;
impl sealed::Sealed for Deterministic {}
impl ReplayMarker for Deterministic {
    const CLASS: ReplayClass = ReplayClass::Deterministic;
}

/// Type-level RetrievedCited.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RetrievedCited;
impl sealed::Sealed for RetrievedCited {}
impl ReplayMarker for RetrievedCited {
    const CLASS: ReplayClass = ReplayClass::RetrievedCited;
}

/// Type-level Composed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Composed;
impl sealed::Sealed for Composed {}
impl ReplayMarker for Composed {
    const CLASS: ReplayClass = ReplayClass::Composed;
}

/// Type-level ModelGenerated.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModelGenerated;
impl sealed::Sealed for ModelGenerated {}
impl ReplayMarker for ModelGenerated {
    const CLASS: ReplayClass = ReplayClass::ModelGenerated;
}

/// Answer value tagged with a type-level replay class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedAnswer<T, R: ReplayMarker> {
    /// Payload.
    pub value: T,
    _replay: PhantomData<R>,
}

impl<T, R: ReplayMarker> TypedAnswer<T, R> {
    /// Construct a typed answer.
    pub fn new(value: T) -> Self {
        Self {
            value,
            _replay: PhantomData,
        }
    }

    /// Wire replay class.
    pub const fn replay_class(&self) -> ReplayClass {
        R::CLASS
    }

    /// Weaken into a weaker (or equal) replay class. Strengthening is a compile-time
    /// or runtime error — use [`weaken_to`] only when `R::CLASS.can_weaken_to(S::CLASS)`.
    pub fn weaken_to<S: ReplayMarker>(self) -> Result<TypedAnswer<T, S>> {
        if R::CLASS.can_weaken_to(S::CLASS) {
            Ok(TypedAnswer::new(self.value))
        } else {
            Err(MolError::ReplayCoercion(format!(
                "cannot coerce {} → {} (strengthening forbidden)",
                R::CLASS,
                S::CLASS
            )))
        }
    }

    /// Erase to wire enum + value (for receipts).
    pub fn erase(self) -> (T, ReplayClass) {
        (self.value, R::CLASS)
    }
}

/// Compile-time: Deterministic may weaken to ModelGenerated.
impl<T> From<TypedAnswer<T, Deterministic>> for TypedAnswer<T, ModelGenerated> {
    fn from(a: TypedAnswer<T, Deterministic>) -> Self {
        TypedAnswer::new(a.value)
    }
}

/// Compile-time: Deterministic may weaken to Composed.
impl<T> From<TypedAnswer<T, Deterministic>> for TypedAnswer<T, Composed> {
    fn from(a: TypedAnswer<T, Deterministic>) -> Self {
        TypedAnswer::new(a.value)
    }
}

/// Compile-time: RetrievedCited may weaken to ModelGenerated.
impl<T> From<TypedAnswer<T, RetrievedCited>> for TypedAnswer<T, ModelGenerated> {
    fn from(a: TypedAnswer<T, RetrievedCited>) -> Self {
        TypedAnswer::new(a.value)
    }
}

// Deliberately NO `From<TypedAnswer<T, ModelGenerated>> for TypedAnswer<T, Deterministic>`.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_cannot_become_deterministic() {
        let m: TypedAnswer<&str, ModelGenerated> = TypedAnswer::new("hallucination");
        let err = m.weaken_to::<Deterministic>().unwrap_err();
        assert!(matches!(err, MolError::ReplayCoercion(_)));
    }

    #[test]
    fn deterministic_may_weaken() {
        let d: TypedAnswer<i32, Deterministic> = TypedAnswer::new(42);
        let c: TypedAnswer<i32, Composed> = d.weaken_to().unwrap();
        assert_eq!(c.value, 42);
        assert_eq!(c.replay_class(), ReplayClass::Composed);
    }

    #[test]
    fn from_impl_weakens() {
        let d = TypedAnswer::<_, Deterministic>::new("ok");
        let m: TypedAnswer<_, ModelGenerated> = d.into();
        assert_eq!(m.replay_class(), ReplayClass::ModelGenerated);
    }
}
