use crate::{CodedInputStream, CodedOutputStream, ProtobufResult, reflect::map::ReflectMap, wire_format::WireType};
use crate::reflect::map::ReflectMapIter;
use crate::reflect::map::ReflectMapIterTrait;
use crate::reflect::runtime_types::RuntimeType;
use crate::reflect::ProtobufValue;
use crate::reflect::ReflectValueBox;
use crate::reflect::ReflectValueRef;
use crate::reflect::RuntimeTypeBox;
use std::collections::hash_map;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
enum Maps {
    U32(HashMap<u32, ReflectValueBox>),
    I32(HashMap<i32, ReflectValueBox>),
    U64(HashMap<u64, ReflectValueBox>),
    I64(HashMap<i64, ReflectValueBox>),
    Bool(HashMap<bool, ReflectValueBox>),
    String(HashMap<String, ReflectValueBox>),
}

impl Maps {
    fn len(&self) -> usize {
        match self {
            Maps::U32(m) => m.len(),
            Maps::I32(m) => m.len(),
            Maps::U64(m) => m.len(),
            Maps::I64(m) => m.len(),
            Maps::Bool(m) => m.len(),
            Maps::String(m) => m.len(),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Maps::U32(m) => m.is_empty(),
            Maps::I32(m) => m.is_empty(),
            Maps::U64(m) => m.is_empty(),
            Maps::I64(m) => m.is_empty(),
            Maps::Bool(m) => m.is_empty(),
            Maps::String(m) => m.is_empty(),
        }
    }

    fn clear(&mut self) {
        match self {
            Maps::U32(m) => m.clear(),
            Maps::I32(m) => m.clear(),
            Maps::U64(m) => m.clear(),
            Maps::I64(m) => m.clear(),
            Maps::Bool(m) => m.clear(),
            Maps::String(m) => m.clear(),
        }
    }

    fn key_type(&self) -> RuntimeTypeBox {
        match self {
            Maps::U32(..) => RuntimeTypeBox::U32,
            Maps::I32(..) => RuntimeTypeBox::I32,
            Maps::U64(..) => RuntimeTypeBox::U64,
            Maps::I64(..) => RuntimeTypeBox::I64,
            Maps::Bool(..) => RuntimeTypeBox::Bool,
            Maps::String(..) => RuntimeTypeBox::String,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DynamicMap {
    value: RuntimeTypeBox,
    maps: Maps,
}

impl DynamicMap {
    pub fn new(key: RuntimeTypeBox, value: RuntimeTypeBox) -> DynamicMap {
        DynamicMap {
            value,
            maps: match key {
                RuntimeTypeBox::U32 => Maps::U32(HashMap::new()),
                RuntimeTypeBox::I32 => Maps::I32(HashMap::new()),
                RuntimeTypeBox::U64 => Maps::U64(HashMap::new()),
                RuntimeTypeBox::I64 => Maps::I64(HashMap::new()),
                RuntimeTypeBox::Bool => Maps::Bool(HashMap::new()),
                RuntimeTypeBox::String => Maps::String(HashMap::new()),
                t => panic!("type cannot be hashmap key: {}", t),
            },
        }
    }

    pub fn compute_size(&self, field_number: u32) -> u32 {
        self.reflect_iter()
            .map(|(k, v)| {
                let entry_len = crate::rt::tag_size(1) + k.to_box().compute_size() + crate::rt::tag_size(2) + v.to_box().compute_size();
                crate::rt::tag_size(field_number) + crate::rt::compute_raw_varint32_size(entry_len) + entry_len
            })
            .sum()
    }

    pub fn write_to_with_cached_sizes(&self, os: &mut CodedOutputStream, field_number: u32) -> ProtobufResult<()> {
        self.reflect_iter()
            .try_for_each(|(k, v)| {
                os.write_tag(field_number, WireType::WireTypeLengthDelimited)?;
                let entry_len = crate::rt::tag_size(1) + k.to_box().compute_size() + crate::rt::tag_size(2) + v.to_box().compute_size();
                os.write_raw_varint32(entry_len)?;

                k.to_box().write_to_with_cached_sizes(os, 1)?;
                v.to_box().write_to_with_cached_sizes(os, 2)
            })
    }

    pub fn merge_from(&mut self, is: &mut CodedInputStream) -> ProtobufResult<()> {
        let len = is.read_raw_varint64()?;
        let old_limit = is.push_limit(len)?;

        let mut key = None;
        let mut value = None;

        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;

            match field_number {
                1 => {
                    let mut keyb = self.key_type().default_value_ref().to_box();
                    keyb.merge_from(is)?;
                    key = Some(keyb)
                },
                2 => {
                    let mut valueb = self.value_type().default_value_ref().to_box();
                    valueb.merge_from(is)?;
                    value = Some(valueb)
                },
                _ => {
                    is.read_unknown(wire_type)?;
                }
            }
        }

        self.insert(key.unwrap(), value.unwrap());

        is.pop_limit(old_limit);
        Ok(())
    }
}

struct DynamicMapIterImpl<'a, K: ProtobufValue + Eq + Hash + 'static> {
    iter: hash_map::Iter<'a, K, ReflectValueBox>,
    value: &'a RuntimeTypeBox,
}

impl<'a, K: ProtobufValue + Eq + Hash + 'static> ReflectMapIterTrait<'a>
    for DynamicMapIterImpl<'a, K>
{
    fn next(&mut self) -> Option<(ReflectValueRef<'a>, ReflectValueRef<'a>)> {
        self.iter
            .next()
            .map(|(k, v)| (K::as_ref(k), v.as_value_ref()))
    }

    fn key_type(&self) -> RuntimeTypeBox {
        K::RuntimeType::runtime_type_box()
    }

    fn value_type(&self) -> RuntimeTypeBox {
        self.value.clone()
    }
}

impl ReflectMap for DynamicMap {
    fn reflect_iter(&self) -> ReflectMapIter {
        match &self.maps {
            Maps::U32(m) => ReflectMapIter::new(DynamicMapIterImpl {
                iter: m.iter(),
                value: &self.value,
            }),
            Maps::I32(m) => ReflectMapIter::new(DynamicMapIterImpl {
                iter: m.iter(),
                value: &self.value,
            }),
            Maps::U64(m) => ReflectMapIter::new(DynamicMapIterImpl {
                iter: m.iter(),
                value: &self.value,
            }),
            Maps::I64(m) => ReflectMapIter::new(DynamicMapIterImpl {
                iter: m.iter(),
                value: &self.value,
            }),
            Maps::Bool(m) => ReflectMapIter::new(DynamicMapIterImpl {
                iter: m.iter(),
                value: &self.value,
            }),
            Maps::String(m) => ReflectMapIter::new(DynamicMapIterImpl {
                iter: m.iter(),
                value: &self.value,
            }),
        }
    }

    fn len(&self) -> usize {
        self.maps.len()
    }

    fn is_empty(&self) -> bool {
        self.maps.is_empty()
    }

    fn get<'a>(&'a self, key: ReflectValueRef) -> Option<ReflectValueRef<'a>> {
        match (&self.maps, key) {
            (Maps::U32(m), ReflectValueRef::U32(v)) => m.get(&v),
            (Maps::U64(m), ReflectValueRef::U64(v)) => m.get(&v),
            (Maps::I32(m), ReflectValueRef::I32(v)) => m.get(&v),
            (Maps::I64(m), ReflectValueRef::I64(v)) => m.get(&v),
            (Maps::Bool(m), ReflectValueRef::Bool(v)) => m.get(&v),
            (Maps::String(m), ReflectValueRef::String(v)) => m.get(&*v),
            _ => None,
        }
        .map(ReflectValueBox::as_value_ref)
    }

    fn insert(&mut self, key: ReflectValueBox, value: ReflectValueBox) {
        assert!(value.get_type() == self.value);
        match (&mut self.maps, &key) {
            (Maps::U32(m), ReflectValueBox::U32(k)) => m.insert(*k, value),
            (Maps::U64(m), ReflectValueBox::U64(k)) => m.insert(*k, value),
            (Maps::I32(m), ReflectValueBox::I32(k)) => m.insert(*k, value),
            (Maps::I64(m), ReflectValueBox::I64(k)) => m.insert(*k, value),
            (Maps::Bool(m), ReflectValueBox::Bool(k)) => m.insert(*k, value),
            (Maps::String(m), _) => match key {
                ReflectValueBox::String(k) => m.insert(k, value),
                _ => panic!("wrong key type"),
            },
            _ => panic!("wrong key type"),
        };
    }

    fn clear(&mut self) {
        self.maps.clear()
    }

    fn key_type(&self) -> RuntimeTypeBox {
        self.maps.key_type()
    }

    fn value_type(&self) -> RuntimeTypeBox {
        self.value.clone()
    }
}
