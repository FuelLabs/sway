script;

use dep_lib::SliceWrapper;

fn main() -> u64 {
    let wrapper = SliceWrapper::empty();
    dep_lib::wrapped_len(wrapper)
}
