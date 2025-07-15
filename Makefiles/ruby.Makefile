.PHONY: setup-ruby
setup-ruby:
	mise run setup

.PHONY: ruby-simple-test
ruby-simple-test:
	mise run simple-test
