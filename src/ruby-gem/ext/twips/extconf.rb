# frozen_string_literal: true

require "mkmf"
require "rb_sys/mkmf"

# https://github.com/oxidize-rb/rb-sys/tree/main/gem
create_rust_makefile("twips/twips_rb")
