# Copyright (c) [2022] SUSE LLC
#
# All Rights Reserved.
#
# This program is free software; you can redistribute it and/or modify it
# under the terms of version 2 of the GNU General Public License as published
# by the Free Software Foundation.
#
# This program is distributed in the hope that it will be useful, but WITHOUT
# ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
# FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for
# more details.
#
# You should have received a copy of the GNU General Public License along
# with this program; if not, contact SUSE LLC.
#
# To contact SUSE LLC about this file by physical or electronic mail, you may
# find current contact information at www.suse.com.

require "yast"
require "agama/dbus/clients/software"

# :nodoc:
module Yast
  # Replacement for the Yast::Package module
  #
  # @see https://github.com/yast/yast-yast2/blob/b8cd178b7f341f6e3438782cb703f4a3ab0529ed/library/packages/src/modules/Package.rb
  class PackageClass < Module
    def main
      puts "Loading mocked module #{__FILE__}"
      @client = Agama::DBus::Clients::Software.new
    end

    # Determines whether the package is available
    #
    # @see https://github.com/yast/yast-yast2/blob/b8cd178b7f341f6e3438782cb703f4a3ab0529ed/library/packages/src/modules/Package.rb#L72
    # @todo Perform a real D-Bus call.
    def Available(_package_name)
      true
    end

    # Determines whether the package is available
    #
    # @todo Perform a real D-Bus call.
    def AvailableAll(_package_names)
      true
    end

    # Determines whether the package is available
    #
    # @see https://github.com/yast/yast-yast2/blob/b8cd178b7f341f6e3438782cb703f4a3ab0529ed/library/packages/src/modules/Package.rb#L121
    # @todo Perform a real D-Bus call.
    def Installed(package_name, target: nil)
      client.package_installed?(package_name)
    end

  private

    attr_reader :client
  end

  Package = PackageClass.new
  Package.main
end
