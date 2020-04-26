use crate::vector::RealVector;
use crate::vector::VectorTarget;

#[repr(C)]
pub struct RustVec<T: VectorTarget<T>> {
    repr: Vec<T>,
}

impl<T: VectorTarget<T>> RustVec<T> {
    pub fn from(v: Vec<T>) -> Self {
        RustVec { repr: v }
    }

    pub fn from_ref(v: &Vec<T>) -> &Self {
        unsafe { std::mem::transmute::<&Vec<T>, &RustVec<T>>(v) }
    }

    pub fn into_vec(self) -> Vec<T> {
        self.repr
    }

    pub fn as_vec(&self) -> &Vec<T> {
        &self.repr
    }

    pub fn as_mut_vec(&mut self) -> &mut Vec<T> {
        &mut self.repr
    }

    pub fn len(&self) -> usize {
        self.repr.len()
    }

    pub fn into_vector(&self, vec: &mut RealVector<T>) {
        for item in &self.repr {
            vec.push_back(item);
        }
    }
}
