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
 * find language contact information at www.suse.com.
 */

import React, { useState } from "react";
import { Button, Form } from "@patternfly/react-core";
import { sprintf } from "sprintf-js";

import { useInstallerClient } from "~/context/installer";
import { _ } from "~/i18n";
import { If, Page, Popup, Section } from "~/components/core";
import { KeymapSelector, LocaleSelector, TimezoneSelector } from "~/components/l10n";
import { noop } from "~/utils";
import { useL10n } from "~/context/l10n";
import { useProduct } from "~/context/product";

/**
 * Popup for selecting a timezone.
 * @component
 *
 * @param {object} props
 * @param {function} props.onFinish - Callback to be called when the timezone is correctly selected.
 * @param {function} props.onCancel - Callback to be called when the timezone selection is canceled.
 */
const TimezonePopup = ({ onFinish = noop, onCancel = noop }) => {
  const { l10n } = useInstallerClient();
  const { timezones, selectedTimezone } = useL10n();

  const [timezoneId, setTimezoneId] = useState(selectedTimezone?.id);
  const { selectedProduct } = useProduct();
  const sortedTimezones = timezones.sort((timezone1, timezone2) => {
    const timezoneText = t => t.parts.join('').toLowerCase();
    return timezoneText(timezone1) > timezoneText(timezone2) ? 1 : -1;
  });

  const onSubmit = async (e) => {
    e.preventDefault();

    if (timezoneId !== selectedTimezone?.id) {
      await l10n.setTimezone(timezoneId);
    }

    onFinish();
  };

  return (
    <Popup
      isOpen
      title={_("Select time zone")}
      description={sprintf(_("%s will use the selected time zone."), selectedProduct.name)}
      blockSize="large"
    >
      <Form id="timezoneForm" onSubmit={onSubmit}>
        <TimezoneSelector value={timezoneId} timezones={sortedTimezones} onChange={setTimezoneId} />
      </Form>
      <Popup.Actions>
        <Popup.Confirm form="timezoneForm" type="submit">
          {_("Accept")}
        </Popup.Confirm>
        <Popup.Cancel onClick={onCancel} />
      </Popup.Actions>
    </Popup>
  );
};

/**
 * Button for opening the selection of timezone.
 * @component
 *
 * @param {object} props
 * @param {React.ReactNode} props.children - Button children.
 */
const TimezoneButton = ({ children }) => {
  const [isPopupOpen, setIsPopupOpen] = useState(false);

  const openPopup = () => setIsPopupOpen(true);
  const closePopup = () => setIsPopupOpen(false);

  return (
    <>
      <Button
        variant="link"
        className="p-0"
        onClick={openPopup}
      >
        {children}
      </Button>

      <If
        condition={isPopupOpen}
        then={
          <TimezonePopup
            isOpen
            onFinish={closePopup}
            onCancel={closePopup}
          />
        }
      />
    </>
  );
};

/**
 * Section for configuring timezone.
 * @component
 */
const TimezoneSection = () => {
  const { selectedTimezone } = useL10n();

  return (
    <Section title={_("Time zone")} icon="schedule">
      <If
        condition={selectedTimezone}
        then={
          <>
            <p>{(selectedTimezone?.parts || []).join(' - ')}</p>
            <TimezoneButton>{_("Change time zone")}</TimezoneButton>
          </>
        }
        else={
          <>
            <p>{_("Time zone not selected yet")}</p>
            <TimezoneButton>{_("Select time zone")}</TimezoneButton>
          </>
        }
      />
    </Section>
  );
};

/**
 * Popup for selecting a locale.
 * @component
 *
 * @param {object} props
 * @param {function} props.onFinish - Callback to be called when the locale is correctly selected.
 * @param {function} props.onCancel - Callback to be called when the locale selection is canceled.
 */
const LocalePopup = ({ onFinish = noop, onCancel = noop }) => {
  const { l10n } = useInstallerClient();
  const { locales, selectedLocales } = useL10n();
  const { selectedProduct } = useProduct();
  const [localeId, setLocaleId] = useState(selectedLocales[0]?.id);

  const sortedLocales = locales.sort((locale1, locale2) => {
    const localeText = l => [l.name, l.territory].join('').toLowerCase();
    return localeText(locale1) > localeText(locale2) ? 1 : -1;
  });

  const onSubmit = async (e) => {
    e.preventDefault();

    const [locale] = selectedLocales;

    if (localeId !== locale?.id) {
      await l10n.setLocales([localeId]);
    }

    onFinish();
  };

  return (
    <Popup
      isOpen
      title={_("Select language")}
      description={sprintf(_("%s will use the selected language."), selectedProduct.name)}
      blockSize="large"
    >
      <Form id="localeForm" onSubmit={onSubmit}>
        <LocaleSelector value={localeId} locales={sortedLocales} onChange={setLocaleId} />
      </Form>
      <Popup.Actions>
        <Popup.Confirm form="localeForm" type="submit">
          {_("Accept")}
        </Popup.Confirm>
        <Popup.Cancel onClick={onCancel} />
      </Popup.Actions>
    </Popup>
  );
};

/**
 * Button for opening the selection of locales.
 * @component
 *
 * @param {object} props
 * @param {React.ReactNode} props.children - Button children.
 */
const LocaleButton = ({ children }) => {
  const [isPopupOpen, setIsPopupOpen] = useState(false);

  const openPopup = () => setIsPopupOpen(true);
  const closePopup = () => setIsPopupOpen(false);

  return (
    <>
      <Button
        variant="link"
        className="p-0"
        onClick={openPopup}
      >
        {children}
      </Button>

      <If
        condition={isPopupOpen}
        then={
          <LocalePopup
            isOpen
            onFinish={closePopup}
            onCancel={closePopup}
          />
        }
      />
    </>
  );
};

/**
 * Section for configuring locales.
 * @component
 */
const LocaleSection = () => {
  const { selectedLocales } = useL10n();

  const [locale] = selectedLocales;

  return (
    <Section title={_("Language")} icon="translate">
      <If
        condition={locale}
        then={
          <>
            <p>{locale?.name} - {locale?.territory}</p>
            <LocaleButton>{_("Change language")}</LocaleButton>
          </>
        }
        else={
          <>
            <p>{_("Language not selected yet")}</p>
            <LocaleButton>{_("Select language")}</LocaleButton>
          </>
        }
      />
    </Section>
  );
};

/**
 * Popup for selecting a keymap.
 * @component
 *
 * @param {object} props
 * @param {function} props.onFinish - Callback to be called when the keymap is correctly selected.
 * @param {function} props.onCancel - Callback to be called when the keymap selection is canceled.
 */
const KeymapPopup = ({ onFinish = noop, onCancel = noop }) => {
  const { l10n } = useInstallerClient();
  const { keymaps, selectedKeymap } = useL10n();
  const { selectedProduct } = useProduct();
  const [keymapId, setKeymapId] = useState(selectedKeymap?.id);

  const sortedKeymaps = keymaps.sort((k1, k2) => k1.name > k2.name ? 1 : -1);

  const onSubmit = async (e) => {
    e.preventDefault();

    if (keymapId !== selectedKeymap?.id) {
      await l10n.setKeymap(keymapId);
    }

    onFinish();
  };

  return (
    <Popup
      isOpen
      title={_("Select keyboard")}
      description={sprintf(_("%s will use the selected keyboard."), selectedProduct.name)}
      blockSize="large"
    >
      <Form id="keymapForm" onSubmit={onSubmit}>
        <KeymapSelector value={keymapId} keymaps={sortedKeymaps} onChange={setKeymapId} />
      </Form>
      <Popup.Actions>
        <Popup.Confirm form="keymapForm" type="submit">
          {_("Accept")}
        </Popup.Confirm>
        <Popup.Cancel onClick={onCancel} />
      </Popup.Actions>
    </Popup>
  );
};

/**
 * Button for opening the selection of keymaps.
 * @component
 *
 * @param {object} props
 * @param {React.ReactNode} props.children - Button children.
 */
const KeymapButton = ({ children }) => {
  const [isPopupOpen, setIsPopupOpen] = useState(false);

  const openPopup = () => setIsPopupOpen(true);
  const closePopup = () => setIsPopupOpen(false);

  return (
    <>
      <Button
        variant="link"
        className="p-0"
        onClick={openPopup}
      >
        {children}
      </Button>

      <If
        condition={isPopupOpen}
        then={
          <KeymapPopup
            isOpen
            onFinish={closePopup}
            onCancel={closePopup}
          />
        }
      />
    </>
  );
};

/**
 * Section for configuring keymaps.
 * @component
 */
const KeymapSection = () => {
  const { selectedKeymap } = useL10n();

  return (
    <Section title={_("Keyboard")} icon="keyboard">
      <If
        condition={selectedKeymap}
        then={
          <>
            <p>{selectedKeymap?.name}</p>
            <KeymapButton>{_("Change keyboard")}</KeymapButton>
          </>
        }
        else={
          <>
            <p>{_("Keyboard not selected yet")}</p>
            <KeymapButton>{_("Select keyboard")}</KeymapButton>
          </>
        }
      />
    </Section>
  );
};

/**
 * Page for configuring localization.
 * @component
 */
export default function L10nPage() {
  return (
    // TRANSLATORS: page title
    <Page icon="globe" title={_("Localization")}>
      <LocaleSection />
      <KeymapSection />
      <TimezoneSection />
    </Page>
  );
}
