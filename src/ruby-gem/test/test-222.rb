#!/usr/bin/env ruby
# frozen_string_literal: true

require "bundler/setup"
require "twips"

puts(Twips::random_scramble_for_event("222"))
