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

import React, { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Button, Text } from "@patternfly/react-core";
import { Icon, AppActions } from "~/components/layout";
import { If, NotificationMark } from "~/components/core";
import { useNotification } from "~/context/notification";

/**
 * Returns siblings for given HTML node
 *
 * @param {HTMLElement} node
 * @returns {HTMLElement[]}
 */
const siblingsFor = (node) => {
  const parent = node?.parentNode;

  return parent ? [...parent.children].filter(n => n !== node) : [];
};

/**
 * Agama sidebar navigation
 * @component
 *
 * @param {object} props
 * @param {React.ReactElement} props.children
 */
export default function Sidebar ({ children }) {
  const [isOpen, setIsOpen] = useState(false);
  const asideRef = useRef(null);
  const closeButtonRef = useRef(null);
  const [notification] = useNotification();

  /**
   * Set siblings as not discoverable and not interactive
   */
  const makeSiblingsInert = () => {
    siblingsFor(asideRef.current).forEach(s => {
      s.setAttribute('inert', '');
      s.setAttribute('aria-hidden', true);
    });
  };

  /**
   * Set siblings as discoverable and interactive
   */
  const makeSiblingsAlive = () => {
    siblingsFor(asideRef.current).forEach(s => {
      s.removeAttribute('inert');
      s.removeAttribute('aria-hidden');
    });
  };

  const open = () => {
    setIsOpen(true);
    makeSiblingsInert();
  };

  const close = () => {
    setIsOpen(false);
    makeSiblingsAlive();
  };

  /**
   * Handler for automatically closing the sidebar when a click bubbles from a
   * children of its content.
   *
   * @param {MouseEvent} event
   */
  const onClick = (event) => {
    const target = event.detail?.originalTarget || event.target;
    const isLinkOrButton = target instanceof HTMLAnchorElement || target instanceof HTMLButtonElement;
    const keepOpen = target.dataset.keepSidebarOpen;

    if (!isLinkOrButton || keepOpen) return;

    close();
  };

  useEffect(() => {
    if (isOpen) closeButtonRef.current.focus();
  }, [isOpen]);

  useLayoutEffect(() => {
    // Ensure siblings do not remain inert when the component is unmount.
    // Using useLayoutEffect over useEffect for allowing the cleanup function to
    // be executed immediately BEFORE unmounting the component and still having
    // access to siblings.
    return () => makeSiblingsAlive();
  }, []);

  // display additional info when running in a development server
  let targetInfo = null;
  if (process.env.WEBPACK_SERVE) {
    let targetUrl = COCKPIT_TARGET_URL;

    // change the localhost URL when connected remotely as it means another machine
    if (COCKPIT_TARGET_URL.includes("localhost") && window.location.hostname !== "localhost") {
      const urlTarget = new URL(COCKPIT_TARGET_URL);
      const url = new URL(window.location);
      url.port = urlTarget.port;
      url.pathname = "/";
      url.search = "";
      url.hash = "";
      targetUrl = url.toString();
    }

    targetInfo = (
      <Text>
        Target server: { " " }
        <Button isInline variant="link" component="a" href={ targetUrl } target="_blank">
          { targetUrl }
        </Button>
      </Text>
    );
  }

  return (
    <>
      <AppActions>
        <button
          onClick={open}
          className="plain-control"
          aria-label="Show global options"
          aria-controls="global-options"
          aria-expanded={isOpen}
        >
          <If
            condition={notification.issues}
            then={<NotificationMark data-variant="sidebar" aria-label="New issues found" />}
          />
          <Icon name="menu" />
        </button>
      </AppActions>

      <aside
        id="global-options"
        ref={asideRef}
        className="wrapper sidebar"
        aria-label="Global options"
        data-state={isOpen ? "visible" : "hidden"}
      >
        <header className="split justify-between">
          <h2>Options</h2>

          <button
            onClick={close}
            ref={closeButtonRef}
            className="plain-control"
            aria-label="Hide navigation and other options"
          >
            <Icon name="menu_open" data-variant="flip-X" onClick={close} />
          </button>
        </header>

        <div className="flex-stack" onClick={onClick}>
          { children }
        </div>

        <footer className="split justify-between" data-state="reversed">
          <a onClick={close}>
            Close <Icon size="16" name="menu_open" data-variant="flip-X" />
          </a>
          { targetInfo }
        </footer>
      </aside>
    </>
  );
}
