/*
 * Copyright (c) [2021-2023] SUSE LLC
 *
 * All Rights Reserved.
 *
 * This program is free software; you can redistribute it and/or modify it
 * under the terms of version 2 of the GNU General Public License as published
 * by the Free Software Foundation.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for
 * more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, contact SUSE LLC.
 *
 * To contact SUSE LLC about this file by physical or electronic mail, you may
 * find current contact information at www.suse.com.
 */

// @ts-check

import { LanguageClient } from "./language";
import { ManagerClient } from "./manager";
import { Monitor } from "./monitor";
import { SoftwareClient } from "./software";
import { StorageClient } from "./storage";
import { UsersClient } from "./users";
import phase from "./phase";
import { QuestionsClient } from "./questions";
import { NetworkClient } from "./network";
import { IssuesClient } from "./issues";
import cockpit from "../lib/cockpit";

const BUS_ADDRESS_FILE = "/run/agama/bus.address";
const MANAGER_SERVICE = "org.opensuse.Agama.Manager1";

/**
 * @typedef {object} InstallerClient
 * @property {LanguageClient} language - language client
 * @property {ManagerClient} manager - manager client
 * @property {Monitor} monitor - service monitor
 * @property {NetworkClient} network - network client
 * @property {SoftwareClient} software - software client
 * @property {StorageClient} storage - storage client
 * @property {UsersClient} users - users client
 * @property {QuestionsClient} questions - questions client
 * @property {IssuesClient} issues - issues client
 * @property {() => Promise<boolean>} isConnected - determines whether the client is connected
 * @property {(handler: () => void) => (() => void)} onDisconnect - registers a handler to run
 *   when the connection is lost. It returns a function to deregister the
 *   handler.
 */

/**
 * Creates the Agama client
 *
 * @return {InstallerClient}
 */
const createClient = (address = "unix:path=/run/agama/bus") => {
  const language = new LanguageClient(address);
  const manager = new ManagerClient(address);
  const monitor = new Monitor(address, MANAGER_SERVICE);
  const network = new NetworkClient();
  const software = new SoftwareClient(address);
  const storage = new StorageClient(address);
  const users = new UsersClient(address);
  const questions = new QuestionsClient(address);
  const issues = new IssuesClient({ storage });

  const isConnected = async () => {
    try {
      await manager.getStatus();
      return true;
    } catch (e) {
      return false;
    }
  };

  return {
    language,
    manager,
    monitor,
    network,
    software,
    storage,
    users,
    questions,
    issues,
    isConnected,
    onDisconnect: (handler) => monitor.onDisconnect(handler)
  };
};

const createDefaultClient = async () => {
  const file = cockpit.file(BUS_ADDRESS_FILE);
  const address = await file.read();
  return (address) ? createClient(address) : createClient();
};

export { createClient, createDefaultClient, phase };
