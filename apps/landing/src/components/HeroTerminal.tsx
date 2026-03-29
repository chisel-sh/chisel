import React from 'react';
import { TerminalDemo, type DemoStep } from './TerminalDemo';

const HERO_DEMOS: DemoStep[] = [
  {
    command: "chisel init",
    output: (
      <pre className="font-mono leading-tight whitespace-pre">
        {`Initializing Chisel workspace for: my-saas-app...
Success! Chisel is ready.

`}<span className="text-gray-500">{`  .chisel/docs/         `}</span>{`Markdown knowledge base
`}<span className="text-gray-500">{`  .chisel/specs/active/         `}</span>{`Draft and in-progress specs
`}<span className="text-gray-500">{`  .chisel/specs/shipped/        `}</span>{`Completed specs
`}<span className="text-gray-500">{`  .chisel/specs/archived/       `}</span>{`Superseded specs

`}<span className="text-[#6EEB83]">{`Try running \`chisel docs\` or \`chisel spec\` to begin.`}</span>
      </pre>
    )
  },
  {
    command: 'chisel spec new "user authentication"',
    output: (
      <pre className="font-mono leading-tight whitespace-pre">
        {`Created spec: User Authentication (user-authentication)
  → .chisel/specs/active/user-authentication.md

`}<span className="text-gray-500">{`---
title: User Authentication
status: draft
created: 2026-03-28
area: auth
---`}</span>{`

## What and Why
## Success Criteria
## Approach
## Open Questions`}
      </pre>
    )
  },
  {
    command: "chisel spec list",
    output: (
      <pre className="font-mono leading-tight whitespace-pre">
        {`┌─────────────────────────────────── SPECS ────────────────────────────────────┐
│ `}<span className="text-gray-500">○</span>{` User Authentication          `}<span className="text-gray-500">draft</span>{`          auth                      │
│ `}<span className="text-[#F2C94C]">◉</span>{` Payment Flow                 `}<span className="text-[#F2C94C]">in-progress</span>{`    payments                  │
│ `}<span className="text-[#6EEB83]">●</span>{` Onboarding V1                `}<span className="text-[#6EEB83]">shipped</span>{`        onboarding               │
└──────────────────────────────────────────────────────────────────────────────┘`}
      </pre>
    )
  },
  {
    command: "chisel --machine spec view payment-flow",
    output: (
      <pre className="font-mono leading-tight whitespace-pre text-gray-400">
        {`slug: payment-flow
title: Payment Flow
status: in-progress
area: payments
created: '2026-03-15'
updated: '2026-03-27'
open_questions:
  - Support Stripe + PayPal or Stripe only?
content: |
  ## What and Why
  Integrate payment processing...`}
      </pre>
    )
  },
  {
    command: 'chisel context create "payments"',
    output: (
      <pre className="font-mono leading-tight whitespace-pre text-gray-400">
        {`<context>
  <spec path=".chisel/specs/active/payment-flow.md">
    ## What and Why
    Integrate payment processing for subscriptions...
  </spec>
  <file path="docs/architecture.md">
    ## Payment Architecture
    Stripe API integration via webhooks...
  </file>
</context>`}
      </pre>
    )
  }
];

export const HeroTerminal: React.FC = () => {
  const footer = (
    <div className="flex gap-4 text-gray-500">
      <span><b className="text-[#EDEDED]">q</b>:quit</span>
      <span><b className="text-[#EDEDED]">/</b>:search</span>
      <span><b className="text-[#EDEDED]">m</b>:machine</span>
      <span><b className="text-[#EDEDED]">?</b>:help</span>
      <span className="ml-auto text-gray-600 italic">hjkl navigation</span>
    </div>
  );

  return <TerminalDemo steps={HERO_DEMOS} title="chisel — demo" footer={footer} />;
};
