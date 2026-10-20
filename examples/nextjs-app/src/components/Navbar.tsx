import React from 'react';
import Image from 'next/image';
import brandSymbol from '@/assets/brand-symbol.svg';

export function Navbar() {
  return (
    <nav className="flex items-center justify-between p-4 bg-slate-900 text-white">
      <div className="flex items-center gap-2">
        <Image src={brandSymbol} alt="Brand Logo" width={32} height={32} />
        <span className="font-bold text-lg">NextApp Vibe</span>
      </div>
      <div className="flex gap-4">
        <a href="#features">Features</a>
        <a href="#pricing">Pricing</a>
      </div>
    </nav>
  );
}
