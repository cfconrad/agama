/*
 * Copyright (c) [2022] SUSE LLC
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

import React, { useState } from "react";
import { Dropdown, DropdownList, DropdownItem, MenuToggle } from '@patternfly/react-core';
import { Icon } from "~/components/layout";
import { useInstallerClient } from "~/context/installer";
import { _ } from "~/i18n";

const KebabToggle = ({ toggleRef, onClick }) => (
  <MenuToggle ref={toggleRef} variant="plain" onClick={onClick} className="kebab-toggler">
    <Icon name="more_vert" size="24" />
  </MenuToggle>
);

export default function WifiNetworkMenu({ settings, position = "right" }) {
  const client = useInstallerClient();
  const [isOpen, setIsOpen] = useState(false);
  const toggle = () => setIsOpen(!isOpen);

  return (
    <Dropdown
      isOpen={isOpen}
      onSelect={toggle}
      toggle={(toggleRef) => <KebabToggle toggleRef={toggleRef} onClick={toggle} />}
      popperProps={{ position }}
      position={position}
    >
      <DropdownList>
        <DropdownItem
          isDanger
          key="forget-network"
          onClick={() => client.network.deleteConnection(settings)}
          icon={<Icon name="delete" size="s" />}
        >
          {/* TRANSLATORS: menu label, remove the selected WiFi network settings */}
          {_("Forget network")}
        </DropdownItem>
      </DropdownList>
    </Dropdown>
  );
}
