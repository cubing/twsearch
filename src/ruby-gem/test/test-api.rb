#!/usr/bin/env ruby
# frozen_string_literal: true

require "bundler/setup"
require "twips"

puts(Twips::random_scramble_for_event("222"))
puts(Twips::derive_scramble_for_event(
  "67002dfc95e6d4288f418fbaa9150aa65b239fd5581f2d067d0293b9321a8b67",
  ["EBNLEND@MABLNHJFHGFEKFIA@DNBKABHHNANA@FD@KKADJAKNFCIJNJGIFCBLEDF", "scrambles", "333", "r1", "g1", "a1", "333", "sub1"],
  "333"
))
