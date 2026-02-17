// @ts-check
import { defineConfig } from 'astro/config';

import tailwindcss from '@tailwindcss/vite';
import react from '@astrojs/react';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
  vite: {
    plugins: [tailwindcss()]
  },

  integrations: [
    react(),
    starlight({
      title: 'Chisel',
      defaultLocale: 'docs',
      locales: {
        docs: {
          label: 'Docs',
          lang: 'en',
        },
      },
      logo: {
        src: './src/assets/logo.svg',
      },
      social: [
        { label: 'GitHub', href: 'https://github.com/chisel-sh/chisel', icon: 'github' },
      ],
      sidebar: [
        {
          label: 'Introduction',
          autogenerate: { directory: 'introduction' },
        },
        {
          label: 'Guide',
          autogenerate: { directory: 'guide' },
        },
        {
          label: 'Reference',
          autogenerate: { directory: 'reference' },
        },
        {
          label: 'Architecture',
          autogenerate: { directory: 'architecture' },
        },
        {
          label: 'Community',
          autogenerate: { directory: 'community' },
        },
        {
          label: 'Tutorials',
          autogenerate: { directory: 'tutorial' },
        },
        {
          label: 'Integrations',
          autogenerate: { directory: 'integrations' },
        },
      ],
      customCss: ['./src/styles/global.css'],
    }),
  ]
});
