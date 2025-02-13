# frozen_string_literal: true

RSpec.describe AppBridge::Account do
  let(:id) { "1" }
  let(:name) { "John Doe" }
  let(:serialized_data) { JSON.generate({ username: "john.doe", password: "foobar" }) }

  subject(:account) do
    AppBridge::Account.new(id, name, serialized_data)
  end

  it { is_expected.to respond_to(:id) }
  it { is_expected.to respond_to(:name) }
  it { is_expected.to respond_to(:serialized_data) }

  it "has an id" do
    expect(account.id).to eq(id)
  end

  it "has a name" do
    expect(account.name).to eq(name)
  end

  it "has serialized data" do
    expect(account.serialized_data).to eq(serialized_data)
  end
end
