import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "GlassScout FM26",
  description: "A revealed-data-only recruitment war room for Football Manager 2026.",
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
