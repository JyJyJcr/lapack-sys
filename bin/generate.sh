#!/bin/bash

set -eux

# * The names of the LAPACK routines in `lapack.h` end with an underscore, which
#   is how they are identified among other functions.
#
# * By default, `char` is converted to `::std::os::raw::c_char`. However, to
#   preserve no_std, it is truncated to `c_char` and gets taken from `libc`.
bindgen --allowlist-function='^.*_$' --use-core --ctypes-prefix crate bin/wrapper.h \
  | sed -e 's/::std::os::raw:://g' \
  | sed -e '/__darwin_size_t/d' \
  | sed -E 's/^([ \t]*)pub fn ([A-Za-z0-9_]+)_\(/\1#[link_name = lapack_func!("\2")]\n\1pub fn \2_(/' \
  > src/lapack.rs

rustfmt src/lapack.rs
