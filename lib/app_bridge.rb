# frozen_string_literal: true

require_relative "app_bridge/version"
begin
  require "app_bridge/#{RUBY_VERSION.split(".").first(2).join(".")}/app_bridge"
rescue LoadError
  require "app_bridge/app_bridge"
end

module AppBridge
  class Error < StandardError; end

  # Represents a trigger event that is recieved from the app.
  class TriggerEvent
    def inspect
      "#<AppBridge::TriggerEvent(id: #{id.inspect}, timestamp: #{timestamp.inspect}, " \
        "serialized_data: #{serialized_data.inspect})>"
    end
  end
end
