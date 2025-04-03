# frozen_string_literal: true

require "timeout"

module AppBridge
  # An app that can be used to fetch events.
  class App
    def fetch_events(context)
      response = request_events_with_timeout(context)

      validate_number_of_events!(response.events)
      validate_store_size!(response.store)

      response
    end

    def polling_timeout
      30 # seconds
    end

    private

    def validate_number_of_events!(events)
      return if events.size <= 100

      raise TooManyEventsError, "Maximum 100 events allowed"
    end

    def validate_store_size!(store)
      return if store.size <= 64 * 1024

      raise StoreTooLargeError, "Store size exceeds 64 kB limit"
    end

    def request_events_with_timeout(context)
      Timeout.timeout(polling_timeout, TimeoutError, "Polling exceeded #{polling_timeout} seconds") do
        _rust_fetch_events(context)
      end
    end
  end
end
