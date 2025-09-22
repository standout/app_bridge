# frozen_string_literal: true

RSpec.describe AppBridge::TriggerContext do
  let(:trigger_id) { "any_trigger" }
  let(:connection) do
    AppBridge::Connection.new("1", "Foobar", "data")
  end
  let(:store) { "store" }

  subject(:trigger_context) do
    AppBridge::TriggerContext.new(trigger_id, connection, store, "{}")
  end

  it { is_expected.to be_a(AppBridge::TriggerContext) }
  it { is_expected.to respond_to(:trigger_id) }
  it { is_expected.to respond_to(:connection) }
  it { is_expected.to respond_to(:store) }

  it "has a trigger_id" do
    expect(trigger_context.trigger_id).to eq(trigger_id)
  end

  it "has a connection" do
    expect(trigger_context.connection.id).to eq(connection.id)
  end

  it "has a store" do
    expect(trigger_context.store).to eq(store)
  end

  it "has serialized_input" do
    expect(trigger_context.serialized_input).to eq("{}")
  end
end
