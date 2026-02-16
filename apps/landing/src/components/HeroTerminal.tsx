import React from 'react';
import { TerminalDemo, type DemoStep } from './TerminalDemo';

const HERO_DEMOS: DemoStep[] = [
  {
    command: "chisel docs list",
    output: (
      <pre className="font-mono leading-tight whitespace-pre">
        {`┌─────────────────────────────── DOCUMENTS ────────────────────────────────────┐
│ 42 documents found                                                           │
├──────────────────────────────────────────────────────────────────────────────┤
│ `}<span className="text-[#4DA3FF]">▸</span>{` embeddings.md                  AI/ML           updated 12m ago             │
│   rag_pipeline.md                RAG             updated 2h ago              │
│   architecture/overview.md       SYSTEM          updated 1d ago              │
└──────────────────────────────────────────────────────────────────────────────┘`}
      </pre>
    )
  },
  {
    command: "chisel observe logs --since 5m",
    output: (
      <pre className="font-mono leading-tight whitespace-pre">
        {`┌─────────────────────────────── LOGS (last 5m) ───────────────────────────────┐
│ 12:04:22  `}<span className="text-[#EB5757]">ERROR</span>{`  db_timeout   Query exceeded 2000ms                          │
│ 12:04:23  `}<span className="text-[#EB5757]">ERROR</span>{`  auth_fail    Invalid token from 10.0.2.14                   │
│ 12:04:45  `}<span className="text-[#F2C94C]">WARN</span>{`   auth_retry   Retrying login for 10.0.2.14                   │
│ 12:05:10  `}<span className="text-[#6EEB83]">INFO</span>{`   auth_success User 42 logged in successfully                 │
│ 12:05:12  `}<span className="text-[#6EEB83]">INFO</span>{`   db_query     SELECT * FROM documents                        │
└──────────────────────────────────────────────────────────────────────────────┘`}
      </pre>
    )
  },
  {
    command: "chisel issues list",
    output: (
      <pre className="font-mono leading-tight whitespace-pre">
        {`┌───────────────────────────────── ISSUES ─────────────────────────────────────┐
│ TODO (3)                     IN PROGRESS (2)                 DONE (1)        │
├──────────────────────────────────────────────────────────────────────────────┤
│ `}<span className="text-[#4DA3FF]">▸</span>{` #42 Add embeddings guide   #51 Improve search perf         #12 Fix typos   │
│   #15 Fix auth leak          #48 TUI alignment fix                           │
│   #55 Netlify forms setup                                                    │
└──────────────────────────────────────────────────────────────────────────────┘`}
      </pre>
    )
  },
  {
    command: "chisel docs summarize embeddings.md --machine",
    output: (
      <pre className="font-mono leading-tight whitespace-pre text-gray-400">
        {`SUMMARY:
- Converts text to vectors
- Enables semantic search
- Required for RAG`}
      </pre>
    )
  },
  {
    command: "chisel observe summarize --window 5m --machine",
    output: (
      <pre className="font-mono leading-tight whitespace-pre text-gray-400">
        {`SUMMARY:
- DB latency spikes (avg 2.1s)
- Repeated auth failures from 10.0.2.14
- Cache miss rate 18%`}
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

  return <TerminalDemo steps={HERO_DEMOS} title="chisel — main_demo" footer={footer} />;
};
