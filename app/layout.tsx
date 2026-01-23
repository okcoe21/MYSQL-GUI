import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "MySQL GUI | Admin Tool",
  description: "Secure, local-only MySQL administration interface",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>
        {children}
      </body>
    </html>
  );
}
