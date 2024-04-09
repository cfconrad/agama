#!/usr/bin/env ruby
# frozen_string_literal: true

# Copyright (c) [2024] SUSE LLC
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
require "autoinstall/script_runner"
require "autoinstall/script"
require "agama/autoyast/users_converter"
require "json"
require "fileutils"
require "pathname"

# :nodoc:
module Agama
  module AutoYaST
    # Converts an AutoYaST profile into an Agama one.
    #
    # It is expected that many of the AutoYaST options are ignored because Agama does not have the
    # same features.
    #
    # The output might include, apart from the JSON Agama profile, a set of scripts (not implemented
    # yet).
    #
    # TODO: handle invalid profiles (YAST_SKIP_XML_VALIDATION).
    # TODO: capture reported errors (e.g., via the Report.Error function).
    class Converter
      # @param profile_url [String] Profile URL
      def initialize(profile_url)
        @profile_url = profile_url
      end

      # Converts the profile into a set of files that Agama can process.
      #
      # @param dir [Pathname,String] Directory to write the profile.
      def to_agama(dir)
        path = Pathname(dir)
        FileUtils.mkdir_p(path)
        import_yast
        profile = read_profile
        File.write(path.join("autoinst.json"), export_profile(profile).to_json)
      end

    private

      attr_reader :profile_url

      def copy_profile; end

      # @return [Hash] AutoYaST profile
      def read_profile
        FileUtils.mkdir_p(Yast::AutoinstConfig.profile_dir)

        # fetch the profile
        Yast::AutoinstConfig.ParseCmdLine(profile_url)
        Yast::ProfileLocation.Process

        # put the profile in the tmp directory
        FileUtils.cp(
          Yast::AutoinstConfig.xml_tmpfile,
          tmp_profile_path
        )

        loop do
          Yast::Profile.ReadXML(tmp_profile_path)
          run_pre_scripts
          break unless File.exist?(Yast::AutoinstConfig.modified_profile)

          FileUtils.cp(Yast::AutoinstConfig.modified_profile, tmp_profile_path)
          FileUtils.rm(Yast::AutoinstConfig.modified_profile)
        end

        Yast::Profile.current
      end

      def run_pre_scripts
        pre_scripts = Yast::Profile.current.fetch_as_hash("scripts")
          .fetch_as_array("pre-scripts")
          .map { |h| Y2Autoinstallation::PreScript.new(h) }
        script_runner = Y2Autoinstall::ScriptRunner.new

        pre_scripts.each do |script|
          script.create_script_file
          script_runner.run(script)
        end
      end

      def tmp_profile_path
        @tmp_profile_path ||= File.join(
          Yast::AutoinstConfig.profile_dir,
          "autoinst.xml"
        )
      end

      # @return [Hash] Agama profile
      def export_profile(profile)
        users = Agama::AutoYaST::UsersConverter.new(profile)
        {
          "software" => export_software(profile.fetch_as_hash("software")),
          "storage"  => export_storage(profile.fetch_as_array("partitioning")),
          "root"     => users.root,
          "user"     => users.user
        }
      end

      # @param drives [Array<Hash>] Array of drives in the AutoYaST partitioning section
      def export_storage(drives)
        # TODO: rely on AutoinstProfile classes
        devices = drives.each_with_object([]) do |d, all|
          next unless d["device"]

          all << d["device"]
        end
        return {} if devices.empty?

        { "bootDevice" => devices.first }
      end

      # @param profile [Hash] Software section from the AutoYaST profile
      def export_software(profile)
        product = profile.fetch_as_array("products").first
        patterns = profile.fetch_as_array("patterns")
        return {} unless product

        { "product" => product, "patterns" => patterns }
      end

      # @param profile [Hash] Users section from the AutoYaST profile
      def export_root(profile)
        users = Agama::AutoYaST::UsersConverter.new(profile)
        users.root
      end

      def import_yast
        Yast.import "AutoinstConfig"
        Yast.import "AutoinstScripts"
        Yast.import "Profile"
        Yast.import "ProfileLocation"
      end
    end
  end
end
