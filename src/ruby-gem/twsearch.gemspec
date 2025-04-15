# frozen_string_literal: true

require_relative "lib/twsearch/version"

Gem::Specification.new do |spec|
  spec.name = "twsearch"
  spec.version = Twsearch::VERSION
  spec.authors = ["Lucas Garron", "Tom Rokicki", "Gregor Billing"]

  spec.summary = "Ruby scramble generation using twsearch"
  spec.description = "Ruby scramble generation using twsearch."
  spec.homepage = "https://github.com/cubing/twsearch"
  spec.required_ruby_version = ">= 3.0.0"
  spec.required_rubygems_version = ">= 3.4.6"

  spec.metadata["bug_tracker_uri"] = "https://github.com/cubing/twsearch/issues"
  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://github.com/cubing/twsearch"

  spec.files = Dir["ext/**/*"] +
               Dir["lib/**/*.rb"] +
               Dir["sig/twsearch.rbs"] +
               Dir["twsearch.gemspec"]
  spec.require_paths = ["lib"]
  spec.extensions = ["ext/twsearch/extconf.rb"]

  spec.metadata["rubygems_mfa_required"] = "true"

  spec.add_dependency("rb_sys", "~> 0.9.117")
end
