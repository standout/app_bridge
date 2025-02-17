# frozen_string_literal: true

require_relative "app_bridge/version"
require_relative "app_bridge/app_bridge"

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
