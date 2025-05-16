# frozen_string_literal: true

RSpec.describe Twsearch do
  it "has a version number" do
    expect(described_class::VERSION).not_to be_nil
  end

  describe ".random_scramble_for_event" do
    subject(:random_scramble_for_event) { described_class.random_scramble_for_event(event_id) }

    context "with 333" do
      let(:event_id) { "333" }

      it { is_expected.not_to be_nil }
    end
  end
end
