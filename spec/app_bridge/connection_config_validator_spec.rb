# frozen_string_literal: true

require "spec_helper"

RSpec.describe AppBridge::ConnectionConfigValidator do
  let(:valid_config) do
    {
      strategies: [
        {
          id: "default",
          type: "api_key",
          storage: %w[api_key]
        }
      ],
      default_strategy: "default",
      runtime: {
        base_url: "https://api.example.com",
        headers: {
          Authorization: { template: "Bearer {{api_key}}" }
        }
      }
    }
  end

  let(:oauth_strategy) do
    {
      id: "default",
      type: "oauth2",
      authorization_server: {
        issuer: "https://oauth.example.com",
        authorization_endpoint: "https://oauth.example.com/authorize",
        token_endpoint: "https://oauth.example.com/token",
        response_types_supported: ["code"]
      },
      oauth_client: {
        client_id: "{{env:OAUTH_CLIENT_ID}}",
        client_secret: "{{env:OAUTH_CLIENT_SECRET}}"
      },
      storage: %w[access_token]
    }
  end

  describe ".validate!" do
    it "accepts a valid oauth2 connection-config" do
      oauth_config = valid_config.merge(
        strategies: [oauth_strategy],
        protected_resource: {
          resource: "https://api.example.com",
          authorization_servers: ["https://oauth.example.com"]
        }
      )

      expect { described_class.validate!(oauth_config) }.not_to raise_error
    end

    it "rejects oauth2 config without authorization_server token_endpoint" do
      invalid = valid_config.merge(
        strategies: [
          oauth_strategy.merge(
            authorization_server: {
              issuer: "https://oauth.example.com",
              authorization_endpoint: "https://oauth.example.com/authorize"
            }
          )
        ]
      )

      expect { described_class.validate!(invalid) }
        .to raise_error(AppBridge::ConnectionConfigError) do |error|
          expect(error.message).to match(/token_endpoint/)
          expect(error.message).not_to match(/\{/)
        end
    end

    it "rejects oauth2 config without authorization_endpoint when grant_types_supported is omitted" do
      invalid = valid_config.merge(
        strategies: [
          oauth_strategy.merge(
            authorization_server: oauth_strategy[:authorization_server].except(:authorization_endpoint)
          )
        ]
      )

      expect { described_class.validate!(invalid) }
        .to raise_error(AppBridge::ConnectionConfigError) do |error|
          expect(error.message).to match(/authorization_endpoint/)
        end
    end

    it "accepts client_credentials-only authorization_server without authorization_endpoint" do
      config = valid_config.merge(
        strategies: [
          oauth_strategy.merge(
            authorization_server: {
              issuer: "https://oauth.example.com",
              token_endpoint: "https://oauth.example.com/token",
              grant_types_supported: ["client_credentials"]
            },
            oauth_client: oauth_strategy[:oauth_client].merge(grant_type: "client_credentials")
          )
        ]
      )

      expect { described_class.validate!(config) }.not_to raise_error
    end

    it "accepts a valid connection-config" do
      expect { described_class.validate!(valid_config) }.not_to raise_error
    end

    it "accepts disabled connection-config" do
      disabled = { strategies: [], default_strategy: "", runtime: {} }

      expect { described_class.validate!(disabled) }.not_to raise_error
    end

    it "raises ConnectionConfigError with path details for invalid config" do
      invalid = valid_config.merge(
        strategies: [{ id: "default", type: "unknown_type" }]
      )

      expect { described_class.validate!(invalid) }
        .to raise_error(AppBridge::ConnectionConfigError) do |error|
          expect(error.message).to include("type")
        end
    end

    it "raises ConnectionConfigError when required top-level keys are missing" do
      expect { described_class.validate!({ strategies: [] }) }
        .to raise_error(AppBridge::ConnectionConfigError, /default_strategy|runtime/)
    end

    it "raises ConnectionConfigError for invalid JSON strings" do
      expect { described_class.validate!("not-json") }
        .to raise_error(AppBridge::ConnectionConfigError, /Invalid connection-config JSON/)
    end

    it "accepts storage as an object with optional titles" do
      object_storage_config = valid_config.merge(
        strategies: [
          {
            id: "default",
            type: "api_key",
            storage: {
              api_key: {},
              expires_at: { title: "Token giltig till" }
            }
          }
        ]
      )

      expect { described_class.validate!(object_storage_config) }.not_to raise_error
    end
  end
end
