// @ts-check
import { defineConfig } from 'astro/config';

import tailwindcss from '@tailwindcss/vite';
import react from '@astrojs/react';
import starlight from '@astrojs/starlight';
import fs from 'node:fs';
import path from 'node:path';

// Parse workspace version from root Cargo.toml
const cargoTomlPath = path.resolve('../../Cargo.toml');
const cargoToml = fs.readFileSync(cargoTomlPath, 'utf-8');
const versionMatch = cargoToml.match(/version = "([^"]+)"/);
const version = versionMatch ? versionMatch[1] : '0.1.0';

// https://astro.build/config
export default defineConfig({
  vite: {
    plugins: [tailwindcss()],
    define: {
      'import.meta.env.CHISEL_VERSION': JSON.stringify(version),
    }
  },

  integrations: [
    react(),
    starlight({
      title: 'Chisel',
      logo: {
        src: './src/assets/logo.svg',
      },
      social: [
        { label: 'GitHub', href: 'https://github.com/chisel-sh/chisel', icon: 'github' },
      ],
      sidebar: [
        {
          label: 'Introduction',
          autogenerate: { directory: 'docs/introduction' },
        },
        {
          label: 'Guide',
          autogenerate: { directory: 'docs/guide' },
        },
        {
          label: 'Reference',
          autogenerate: { directory: 'docs/reference' },
        },
        {
          label: 'Architecture',
          autogenerate: { directory: 'docs/architecture' },
        },
        {
          label: 'Community',
          autogenerate: { directory: 'docs/community' },
        },
        {
          label: 'Tutorials',
          autogenerate: { directory: 'docs/tutorial' },
        },
        {
          label: 'Integrations',
          autogenerate: { directory: 'docs/integrations' },
        },
      ],
      customCss: ['./src/styles/global.css'],
    }),
  ]
});
