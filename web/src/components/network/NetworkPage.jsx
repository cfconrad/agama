/*
 * Copyright (c) [2023] SUSE LLC
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

import React, { useEffect, useState } from "react";
import { Button, Skeleton } from "@patternfly/react-core";
import { Icon } from "~/components/layout";
import { useInstallerClient } from "~/context/installer";
import { If, Page, Section } from "~/components/core";
import { ConnectionsTable, IpSettingsForm, NetworkPageMenu, WifiSelector } from "~/components/network";
import { ConnectionTypes, NetworkEventTypes } from "~/client/network";
import { _ } from "~/i18n";

/**
 * Internal component for displaying info when none wire connection is found
 * @component
 */
const NoWiredConnections = () => {
  return (
    <div className="stack">
      <div>{_("No wired connections found.")}</div>
    </div>
  );
};

/**
 * Internal component for displaying info when none WiFi connection is found
 * @component
 *
 * @param {object} props
 * @param {boolean} props.supported - whether the system supports scanning WiFi networks
 * @param {boolean} props.openWifiSelector - the function for opening the WiFi selector
 */
const NoWifiConnections = ({ wifiScanSupported, openWifiSelector }) => {
  const message = wifiScanSupported
    ? _("The system has not been configured for connecting to a WiFi network yet.")
    : _("The system does not support WiFi connections, probably because of missing or disabled hardware.");

  return (
    <div className="stack">
      <div>{_("No WiFi connections found.")}</div>
      <div>{message}</div>
      <If
        condition={wifiScanSupported}
        then={
          <>
            <Button
              variant="primary"
              onClick={openWifiSelector}
              icon={<Icon name="wifi_find" size="s" />}
            >
              {/* TRANSLATORS: button label */}
              {_("Connect to a Wi-Fi network")}
            </Button>
          </>
        }
      />
    </div>
  );
};

/**
 * Page component holding Network settings
 * @component
 */
export default function NetworkPage() {
  const { network: client } = useInstallerClient();
  const [connections, setConnections] = useState(undefined);
  const [devices, setDevices] = useState(undefined);
  const [selectedConnection, setSelectedConnection] = useState(null);
  const [wifiScanSupported, setWifiScanSupported] = useState(false);
  const [wifiSelectorOpen, setWifiSelectorOpen] = useState(false);

  const openWifiSelector = () => setWifiSelectorOpen(true);
  const closeWifiSelector = () => setWifiSelectorOpen(false);

  useEffect(() => {
    return client.onNetworkChange(({ type, payload }) => {
      switch (type) {
        case NetworkEventTypes.DEVICE_ADDED: {
          setDevices((devs) => {
            const newDevices = devs.filter((d) => d.name !== payload.name);
            return [...newDevices, client.fromApiDevice(payload)];
          });
          break;
        }

        case NetworkEventTypes.DEVICE_UPDATED: {
          const [name, data] = payload;
          setDevices(devs => {
            const newDevices = devs.filter((d) => d.name !== name);
            return [...newDevices, client.fromApiDevice(data)];
          });
          break;
        }

        case NetworkEventTypes.DEVICE_REMOVED: {
          setDevices(devs => devs.filter((d) => d.name !== payload));
          break;
        }
      }
      client.connections().then(setConnections);
    });
  }, [client, devices]);

  useEffect(() => {
    if (connections !== undefined) return;

    client.settings().then((s) => setWifiScanSupported(s.wireless_enabled));
    client.connections().then(setConnections);
  }, [client, connections]);

  useEffect(() => {
    if (devices !== undefined) return;

    client.devices().then(setDevices);
  }, [client, devices]);

  const selectConnection = ({ id }) => {
    client.getConnection(id).then(setSelectedConnection);
  };

  const forgetConnection = async ({ id }) => {
    await client.deleteConnection(id);
    setConnections(undefined);
  };

  const updateConnections = async () => {
    setConnections(undefined);
    setDevices(undefined);
  };

  const ready = (connections !== undefined) && (devices !== undefined);

  const WifiConnections = () => {
    const wifiConnections = connections.filter(c => c.wireless);

    if (wifiConnections.length === 0) {
      return (
        <NoWifiConnections wifiScanSupported={wifiScanSupported} openWifiSelector={openWifiSelector} />
      );
    }

    return (
      <ConnectionsTable connections={wifiConnections} devices={devices} onEdit={selectConnection} onForget={forgetConnection} />
    );
  };

  const WiredConnections = () => {
    const wiredConnections = connections.filter(c => !c.wireless && (c.id !== "lo"));

    if (wiredConnections.length === 0) return <NoWiredConnections />;

    return <ConnectionsTable connections={wiredConnections} devices={devices} onEdit={selectConnection} />;
  };

  return (
    // TRANSLATORS: page title
    <Page icon="settings_ethernet" title={_("Network")}>
      { /* TRANSLATORS: page section */}
      <Section title={_("Wired networks")} icon="lan">
        {ready ? <WiredConnections /> : <Skeleton />}
      </Section>

      { /* TRANSLATORS: page section */}
      <Section title={_("WiFi networks")} icon="wifi">
        {ready ? <WifiConnections /> : <Skeleton />}
      </Section>

      <NetworkPageMenu wifiScanSupported={wifiScanSupported} openWifiSelector={openWifiSelector} />

      <If
        condition={wifiScanSupported}
        then={<WifiSelector isOpen={wifiSelectorOpen} onClose={closeWifiSelector} />}
      />

      { /* TODO: improve the connections edition */}
      <If
        condition={selectedConnection}
        then={<IpSettingsForm connection={selectedConnection} onClose={() => setSelectedConnection(null)} onSubmit={updateConnections} />}
      />
    </Page>
  );
}
