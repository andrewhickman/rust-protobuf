use crate::{CodedInputStream, CodedOutputStream, ProtobufResult, reflect::repeated::ReflectRepeated};
use crate::reflect::repeated::ReflectRepeatedIter;
use crate::reflect::ReflectValueBox;
use crate::reflect::ReflectValueRef;
use crate::reflect::RuntimeTypeBox;

#[derive(Debug, Clone)]
pub(crate) struct DynamicRepeated {
    elem: RuntimeTypeBox,
    vec: Vec<ReflectValueBox>,
}

impl ReflectRepeated for DynamicRepeated {
    fn reflect_iter(&self) -> ReflectRepeatedIter {
        unimplemented!()
    }

    fn len(&self) -> usize {
        self.vec.len()
    }

    fn get(&self, index: usize) -> ReflectValueRef {
        self.vec[index].as_value_ref()
    }

    fn set(&mut self, index: usize, value: ReflectValueBox) {
        assert_eq!(self.elem, value.get_type());
        self.vec[index] = value;
    }

    fn push(&mut self, value: ReflectValueBox) {
        assert_eq!(self.elem, value.get_type());
        self.vec.push(value);
    }

    fn clear(&mut self) {
        self.vec.clear();
    }

    fn element_type(&self) -> RuntimeTypeBox {
        self.elem.clone()
    }
}

impl DynamicRepeated {
    pub fn new(elem: RuntimeTypeBox) -> DynamicRepeated {
        DynamicRepeated {
            elem,
            vec: Vec::new(),
        }
    }
    
    pub fn compute_size(&self, field_number: u32) -> u32 { 
        self.vec.iter().map(|v| {
            crate::rt::tag_size(field_number) + v.compute_size()
        }).sum()
    }

    pub fn write_to_with_cached_sizes(&self, os: &mut CodedOutputStream, field_number: u32) -> ProtobufResult<()> {
        self.vec.iter().try_for_each(|v| {
            v.write_to_with_cached_sizes(os, field_number)
        })
    }

    pub fn merge_from(&mut self, is: &mut CodedInputStream) -> ProtobufResult<()> {
        let mut value = self.elem.default_value_ref().to_box();
        value.merge_from(is)?;

        self.vec.push(value);

        Ok(())
    }
}
