// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use crate::array::{
    ArrayBuilder, ArrayRef, GenericBinaryArray, GenericListBuilder, OffsetSizeTrait,
    UInt8Builder,
};
use crate::error::Result;
use std::any::Any;
use std::sync::Arc;

///  Array builder for `BinaryArray`
#[derive(Debug)]
pub struct GenericBinaryBuilder<OffsetSize: OffsetSizeTrait> {
    builder: GenericListBuilder<OffsetSize, UInt8Builder>,
}

impl<OffsetSize: OffsetSizeTrait> GenericBinaryBuilder<OffsetSize> {
    /// Creates a new `GenericBinaryBuilder`, `capacity` is the number of bytes in the values
    /// array
    pub fn new(capacity: usize) -> Self {
        let values_builder = UInt8Builder::new(capacity);
        Self {
            builder: GenericListBuilder::new(values_builder),
        }
    }

    /// Creates a new `GenericBinaryBuilder`,
    /// `data_capacity` is the number of bytes of string data to pre-allocate space for in this builder
    /// `item_capacity` is the number of items to pre-allocate space for in this builder
    pub fn with_capacity(item_capacity: usize, data_capacity: usize) -> Self {
        let values_builder = UInt8Builder::new(data_capacity);
        Self {
            builder: GenericListBuilder::with_capacity(values_builder, item_capacity),
        }
    }

    /// Appends a single byte value into the builder's values array.
    ///
    /// Note, when appending individual byte values you must call `append` to delimit each
    /// distinct list value.
    #[inline]
    pub fn append_byte(&mut self, value: u8) -> Result<()> {
        self.builder.values().append_value(value)?;
        Ok(())
    }

    /// Appends a byte slice into the builder.
    ///
    /// Automatically calls the `append` method to delimit the slice appended in as a
    /// distinct array element.
    #[inline]
    pub fn append_value(&mut self, value: impl AsRef<[u8]>) -> Result<()> {
        self.builder.values().append_slice(value.as_ref())?;
        self.builder.append(true)?;
        Ok(())
    }

    /// Finish the current variable-length list array slot.
    #[inline]
    pub fn append(&mut self, is_valid: bool) -> Result<()> {
        self.builder.append(is_valid)
    }

    /// Append a null value to the array.
    #[inline]
    pub fn append_null(&mut self) -> Result<()> {
        self.append(false)
    }

    /// Builds the `BinaryArray` and reset this builder.
    pub fn finish(&mut self) -> GenericBinaryArray<OffsetSize> {
        GenericBinaryArray::<OffsetSize>::from(self.builder.finish())
    }
}

impl<OffsetSize: OffsetSizeTrait> ArrayBuilder for GenericBinaryBuilder<OffsetSize> {
    /// Returns the number of array slots in the builder
    fn len(&self) -> usize {
        self.builder.len()
    }

    /// Returns whether the number of array slots is zero
    fn is_empty(&self) -> bool {
        self.builder.is_empty()
    }

    /// Builds the array and reset this builder.
    fn finish(&mut self) -> ArrayRef {
        Arc::new(self.finish())
    }

    /// Returns the builder as a non-mutable `Any` reference.
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Returns the builder as a mutable `Any` reference.
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    /// Returns the boxed builder as a box of `Any`.
    fn into_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl<Ptr, OffsetSize: OffsetSizeTrait> FromIterator<Option<Ptr>>
    for GenericBinaryBuilder<OffsetSize>
where
    Ptr: AsRef<[u8]>,
{
    fn from_iter<I: IntoIterator<Item = Option<Ptr>>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();
        let size_hint = upper.unwrap_or(lower);

        let mut builder = GenericBinaryBuilder::with_capacity(size_hint, 0);

        iter.for_each(|item| {
            if let Some(a) = item {
                builder
                    .append_value(a.as_ref())
                    .expect("Unable to append a value to a binary array builder.");
            } else {
                builder
                    .append_null()
                    .expect("Unable to append a null value to a binary array builder.");
            }
        });

        builder
    }
}

#[cfg(test)]
mod tests {
    use crate::array::builder::{BinaryBuilder, LargeBinaryBuilder};
    use crate::array::Array;

    #[test]
    fn test_binary_array_builder() {
        let mut builder = BinaryBuilder::new(20);

        builder.append_byte(b'h').unwrap();
        builder.append_byte(b'e').unwrap();
        builder.append_byte(b'l').unwrap();
        builder.append_byte(b'l').unwrap();
        builder.append_byte(b'o').unwrap();
        builder.append(true).unwrap();
        builder.append(true).unwrap();
        builder.append_byte(b'w').unwrap();
        builder.append_byte(b'o').unwrap();
        builder.append_byte(b'r').unwrap();
        builder.append_byte(b'l').unwrap();
        builder.append_byte(b'd').unwrap();
        builder.append(true).unwrap();

        let binary_array = builder.finish();

        assert_eq!(3, binary_array.len());
        assert_eq!(0, binary_array.null_count());
        assert_eq!([b'h', b'e', b'l', b'l', b'o'], binary_array.value(0));
        assert_eq!([] as [u8; 0], binary_array.value(1));
        assert_eq!([b'w', b'o', b'r', b'l', b'd'], binary_array.value(2));
        assert_eq!(5, binary_array.value_offsets()[2]);
        assert_eq!(5, binary_array.value_length(2));
    }

    #[test]
    fn test_large_binary_array_builder() {
        let mut builder = LargeBinaryBuilder::new(20);

        builder.append_byte(b'h').unwrap();
        builder.append_byte(b'e').unwrap();
        builder.append_byte(b'l').unwrap();
        builder.append_byte(b'l').unwrap();
        builder.append_byte(b'o').unwrap();
        builder.append(true).unwrap();
        builder.append(true).unwrap();
        builder.append_byte(b'w').unwrap();
        builder.append_byte(b'o').unwrap();
        builder.append_byte(b'r').unwrap();
        builder.append_byte(b'l').unwrap();
        builder.append_byte(b'd').unwrap();
        builder.append(true).unwrap();

        let binary_array = builder.finish();

        assert_eq!(3, binary_array.len());
        assert_eq!(0, binary_array.null_count());
        assert_eq!([b'h', b'e', b'l', b'l', b'o'], binary_array.value(0));
        assert_eq!([] as [u8; 0], binary_array.value(1));
        assert_eq!([b'w', b'o', b'r', b'l', b'd'], binary_array.value(2));
        assert_eq!(5, binary_array.value_offsets()[2]);
        assert_eq!(5, binary_array.value_length(2));
    }
}