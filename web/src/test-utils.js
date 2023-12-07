/*
 * Copyright (c) [2022-2023] SUSE LLC
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

/**
 * A module for providing utility functions for testing
 *
 * @module test-utils
 */

import React from "react";
import { MemoryRouter } from "react-router-dom";
import userEvent from "@testing-library/user-event";
import { render } from "@testing-library/react";

import { createClient } from "~/client/index";
import { InstallerClientProvider } from "~/context/installer";
import { NotificationProvider } from "~/context/notification";
import { Layout } from "~/components/layout";
import { noop } from "./utils";
import cockpit from "./lib/cockpit";
import { InstallerL10nProvider } from "./context/installerL10n";
import { L10nProvider } from "./context/l10n";

/**
 * Internal mock for manipulating routes, using ["/"] by default
 */
const initialRoutes = jest.fn().mockReturnValue(["/"]);

/**
 * Allows checking when react-router-dom navigate function  was
 * called with certain path
 *
 * @example
 *   expect(mockNavigateFn).toHaveBeenCalledWith("/")
 */
const mockNavigateFn = jest.fn();

/**
 * Allows manipulating MemoryRouter routes for testing purpose
 *
 * NOTE: on purpose, it will take effect only once.
 *
 * @example
 *   mockRoutes("/products", "/storage");
 *
 * @param {...string} routes
 */
const mockRoutes = (...routes) => initialRoutes.mockReturnValueOnce(routes);

// Centralize the react-router-dom mock here
jest.mock('react-router-dom', () => ({
  ...jest.requireActual("react-router-dom"),
  useNavigate: () => mockNavigateFn,
  Navigate: ({ to: route }) => <>Navigating to {route}</>,
  Outlet: () => <>Outlet Content</>
}));

const Providers = ({ children, withL10n }) => {
  const client = createClient();

  // FIXME: workaround to fix the tests. We should inject
  // the client instead of mocking `createClient`.
  if (!client.onDisconnect) {
    client.onDisconnect = noop;
  }

  if (withL10n) {
    return (
      <InstallerClientProvider client={client}>
        <InstallerL10nProvider>
          <L10nProvider>
            {children}
          </L10nProvider>
        </InstallerL10nProvider>
      </InstallerClientProvider>
    );
  }

  return (
    <InstallerClientProvider client={client}>
      {children}
    </InstallerClientProvider>
  );
};

const installerRender = (ui, options = {}) => {
  const Wrapper = ({ children }) => (
    <Providers withL10n={options.withL10n}>
      <MemoryRouter initialEntries={initialRoutes()}>
        <Layout>{children}</Layout>
      </MemoryRouter>
    </Providers>
  );

  return (
    {
      user: userEvent.setup(),
      ...render(ui, { wrapper: Wrapper, ...options })
    }
  );
};

// Add an option to include or not the layout.
const plainRender = (ui, options = {}) => {
  const { layout, ...opts } = options;
  if (layout) {
    opts.wrapper = Layout;
  }
  return (
    {
      user: userEvent.setup(),
      ...render(ui, opts)
    }
  );
};

/**
 * Creates a function to register callbacks
 *
 * It can be useful to mock functions that might receive a callback that you can
 * execute on-demand during the test.
 *
 * @return a tuple with the mocked function and the list of callbacks.
 */
const createCallbackMock = () => {
  const callbacks = [];
  const on = (callback) => {
    callbacks.push(callback);
    return () => {
      const position = callbacks.indexOf(callback);
      if (position > -1) callbacks.splice(position, 1);
    };
  };
  return [on, callbacks];
};

/**
 * Wraps the content with a notification provider
 *
 * @param {React.ReactNode} content
 * @returns {React.ReactNode}
 */
const withNotificationProvider = (content) => {
  return (
    <NotificationProvider>
      {content}
    </NotificationProvider>
  );
};

/**
 * Mocks the cockpit.gettext() method with an identity function (returns
 * the original untranslated text)
 */
const mockGettext = () => {
  const gettextFn = jest.fn();
  gettextFn.mockImplementation((text) => {
    return text;
  });

  cockpit.gettext.mockImplementation(gettextFn);
};

export {
  plainRender,
  installerRender,
  createCallbackMock,
  mockGettext,
  mockNavigateFn,
  mockRoutes,
  withNotificationProvider
};
