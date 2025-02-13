# frozen_string_literal: true

RSpec.describe AppBridge::TriggerContext do
  let(:trigger_id) { "any_trigger" }
  let(:account) do
    AppBridge::Account.new("1", "Foobar", "data")
  end
  let(:store) { "store" }

  subject(:trigger_context) do
    AppBridge::TriggerContext.new(trigger_id, account, store)
  end

  it { is_expected.to be_a(AppBridge::TriggerContext) }
  it { is_expected.to respond_to(:trigger_id) }
  it { is_expected.to respond_to(:account) }
  it { is_expected.to respond_to(:store) }

  it "has a trigger_id" do
    expect(trigger_context.trigger_id).to eq(trigger_id)
  end

  it "has an account" do
    expect(trigger_context.account.id).to eq(account.id)
  end

  it "has a store" do
    expect(trigger_context.store).to eq(store)
  end
end
