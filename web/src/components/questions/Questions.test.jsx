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

import React from "react";

import { act, screen, waitFor } from "@testing-library/react";
import { installerRender } from "~/test-utils";
import { createClient } from "~/client";

import { Questions } from "~/components/questions";

jest.mock("~/client");
jest.mock("~/components/questions/GenericQuestion", () => () => <div>A Generic question mock</div>);
jest.mock("~/components/questions/LuksActivationQuestion", () => () => <div>A LUKS activation question mock</div>);

const handlers = {};
const genericQuestion = { id: 1, type: 'generic' };
const luksActivationQuestion = { id: 1, class: "storage.luks_activation" };
let pendingQuestions = [];

beforeEach(() => {
  createClient.mockImplementation(() => {
    return {
      questions: {
        getQuestions: () => Promise.resolve(pendingQuestions),
        // Capture the handler for the onQuestionAdded signal for triggering it manually
        onQuestionAdded: onAddHandler => {
          handlers.onAdd = onAddHandler;
          return jest.fn;
        },
        // Capture the handler for the onQuestionREmoved signal for triggering it manually
        onQuestionRemoved: onRemoveHandler => {
          handlers.onRemove = onRemoveHandler;
          return jest.fn;
        },
      }
    };
  });
});

describe("Questions", () => {
  describe("when there are no pending questions", () => {
    beforeEach(() => {
      pendingQuestions = [];
    });

    it("renders nothing", async () => {
      installerRender(<Questions />);

      const main = await screen.findByRole("main");
      await waitFor(() => expect(main).toBeEmptyDOMElement());
    });
  });

  describe("when a new question is added", () => {
    it("push it into the pending queue", async () => {
      installerRender(<Questions />);

      const main = await screen.findByRole("main");
      await waitFor(() => expect(main).toBeEmptyDOMElement());

      // Manually triggers the handler given for the onQuestionAdded signal
      act(() => handlers.onAdd(genericQuestion));

      await screen.findByText("A Generic question mock");
    });
  });

  describe("when a question is removed", () => {
    beforeEach(() => {
      pendingQuestions = [genericQuestion];
    });

    it("removes it from the queue", async () => {
      installerRender(<Questions />);
      await screen.findByText("A Generic question mock");

      // Manually triggers the handler given for the onQuestionRemoved signal
      act(() => handlers.onRemove(genericQuestion.id));

      const content = screen.queryByText("A Generic question mock");
      expect(content).toBeNull();
    });
  });

  describe("when there is a generic question pending", () => {
    beforeEach(() => {
      pendingQuestions = [genericQuestion];
    });

    it("renders a GenericQuestion component", async () => {
      installerRender(<Questions />);

      await screen.findByText("A Generic question mock");
    });
  });

  describe("when there is a LUKS activation question pending", () => {
    beforeEach(() => {
      pendingQuestions = [luksActivationQuestion];
    });

    it("renders a LuksActivationQuestion component", async () => {
      installerRender(<Questions />);

      await screen.findByText("A LUKS activation question mock");
    });
  });
});
