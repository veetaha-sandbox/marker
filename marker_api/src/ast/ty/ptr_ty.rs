use crate::{
    ast::{generic::SemLifetime, Mutability},
    ffi::FfiOption,
};

use super::SemTy;

/// The semantic representation of a reference like [`&T`](prim@reference)
/// or [`&mut T`](prim@reference)
#[repr(C)]
#[derive(Debug)]
pub struct SemRefTy<'ast> {
    lifetime: FfiOption<SemLifetime<'ast>>,
    mutability: Mutability,
    inner_ty: SemTy<'ast>,
}

impl<'ast> SemRefTy<'ast> {
    pub fn has_lifetime(&self) -> bool {
        self.lifetime.get().is_some()
    }

    pub fn lifetime(&self) -> Option<&SemLifetime<'ast>> {
        self.lifetime.get()
    }

    /// This returns the [`Mutability`] of the referenced type.
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    /// This returns the inner [`SemTy`]
    pub fn inner_ty(&self) -> &SemTy<'ast> {
        &self.inner_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemRefTy<'ast> {
    pub fn new(lifetime: Option<SemLifetime<'ast>>, mutability: Mutability, inner_ty: SemTy<'ast>) -> Self {
        Self {
            lifetime: lifetime.into(),
            mutability,
            inner_ty,
        }
    }
}

/// The semantic representation of a raw pointer like [`*const T`](prim@pointer)
/// or [`*mut T`](prim@pointer)
#[repr(C)]
#[derive(Debug)]
pub struct SemRawPtrTy<'ast> {
    mutability: Mutability,
    inner_ty: SemTy<'ast>,
}

impl<'ast> SemRawPtrTy<'ast> {
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    pub fn inner_ty(&self) -> &SemTy<'ast> {
        &self.inner_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemRawPtrTy<'ast> {
    pub fn new(mutability: Mutability, inner_ty: SemTy<'ast>) -> Self {
        Self { mutability, inner_ty }
    }
}
