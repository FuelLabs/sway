#! /bin/bash

# Nedds to exist at least one line between them
remove_generated_code() {
    START=`grep -n "BEGIN $1" ./src/codec.sw`
    START=${START%:*}
    END=`grep -n "END $1" ./src/codec.sw`
    END=${END%:*}
    sed -i "$((START+1)),$((END-1))d" ./src/codec.sw
}

remove_generated_code "STRARRAY_ENCODE"
START=1
END=16
for ((i=END;i>=START;i--)); do
    CODE="impl AbiEncode for str[$i] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }"
    sed -i "s/\/\/ BEGIN STRARRAY_ENCODE/\/\/ BEGIN STRARRAY_ENCODE\n$CODE/g" ./src/codec.sw
done

remove_generated_code "STRARRAY_DECODE"
START=1
END=16
for ((i=END;i>=START;i--)); do
    CODE="impl AbiDecode for str[$i] { fn abi_decode(ref mut buffer: BufferReader) -> str[$i] { let data = buffer.read_bytes($i); asm(s: data.ptr()) { s: str[$i] } } }"
    sed -i "s/\/\/ BEGIN STRARRAY_DECODE/\/\/ BEGIN STRARRAY_DECODE\n$CODE/g" ./src/codec.sw
done
