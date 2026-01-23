import { NextRequest, NextResponse } from "next/server";
import { decrypt } from "@/lib/session";

const protectedRoutes = ["/dashboard", "/api/databases", "/api/tables", "/api/data", "/api/query"];
const publicRoutes = ["/login", "/api/auth"];

export default async function middleware(req: NextRequest) {
    const path = req.nextUrl.pathname;
    const isProtectedRoute = protectedRoutes.some(route => path.startsWith(route));
    const isPublicRoute = publicRoutes.some(route => path.startsWith(route));

    // 1. Decrypt the session from the cookie
    const cookie = req.cookies.get("session")?.value;
    const session = cookie ? await decrypt(cookie).catch(() => null) : null;

    // 2. Redirect to /login if the user is not authenticated and trying to access a protected route
    if (isProtectedRoute && !session) {
        return NextResponse.redirect(new URL("/login", req.nextUrl));
    }

    // 3. Redirect to /dashboard if the user is authenticated and trying to access a public route
    if (isPublicRoute && session && !path.startsWith("/api/auth")) {
        return NextResponse.redirect(new URL("/dashboard", req.nextUrl));
    }

    return NextResponse.next();
}

// Routes Middleware should not run on
export const config = {
    matcher: ["/((?!_next/static|_next/image|favicon.ico).*)"],
};
