import React from 'react';
import Image from 'next/image';
import { Navbar } from '@/components/Navbar';
import './globals.css';

export default function Home() {
  return (
    <main className="min-h-screen bg-slate-950 text-white">
      <Navbar />
      <section className="max-w-4xl mx-auto py-16 px-4 text-center">
        <h1 className="text-4xl font-extrabold mb-4">
          Ship Faster with Vibe-Coded Next.js
        </h1>
        <p className="text-slate-400 mb-8">
          The ultimate dashboard for modern engineers.
        </p>
        <div className="rounded-xl overflow-hidden shadow-2xl border border-slate-800">
          <Image
            src="/images/dashboard-preview.png"
            alt="Dashboard UI Preview"
            width={1200}
            height={675}
            priority
          />
        </div>
      </section>
    </main>
  );
}
