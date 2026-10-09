import type { Metadata } from "next";
import { WalletProvider } from "@/context/WalletContext";
import "./globals.css";

export const metadata: Metadata = {
  title: "Rain USDR Dashboard",
  description: "Frontend for the Rain USDR Soroban smart contract suite",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="h-full scroll-smooth">
      <body className="min-h-full flex flex-col bg-stellar-dark text-stellar-foreground">
        <WalletProvider>{children}</WalletProvider>
      </body>
    </html>
  );
}
