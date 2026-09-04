//! Java mapping for `Ordering`.

use portable_build::Ordering;

use super::support::java_intrinsic_mapping;

java_intrinsic_mapping!(JavaOrdering, Ordering);
