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

import React from "react";
import { screen } from "@testing-library/react";
import { plainRender } from "~/test-utils";
import LanguageSwitcher from "./LanguageSwitcher";

const mockLanguage = "es-es";
let mockChangeLanguageFn;

jest.mock("~/lib/cockpit", () => ({
  gettext: term => term,
  manifests: {
    agama: {
      locales: {
        "de-de": "Deutsch",
        "en-us": "English (US)",
        "es-es": "Español"
      }
    }
  }
}));

jest.mock("~/context/l10n", () => ({
  ...jest.requireActual("~/context/l10n"),
  useL10n: () => ({
    language: mockLanguage,
    changeLanguage: mockChangeLanguageFn
  })
}));

beforeEach(() => {
  mockChangeLanguageFn = jest.fn();
});

it("LanguageSwitcher", async () => {
  const { user } = plainRender(<LanguageSwitcher />);
  expect(screen.getByRole("option", { name: "Español" }).selected).toBe(true);
  await user.selectOptions(
    screen.getByRole("combobox", { label: "Display Language" }),
    screen.getByRole("option", { name: "English (US)" })
  );
  expect(mockChangeLanguageFn).toHaveBeenCalledWith("en-us");
});
