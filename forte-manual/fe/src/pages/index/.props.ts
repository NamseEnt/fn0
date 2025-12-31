// Auto-generated from src/pages/index/mod.rs

export type Props = { Ok: { user?: { username: string, avatarUrl?: string, notifications: number }, banners: { id: string, title: string, link: string, variant: "Primary" | "Secondary" | "Alert" }[], feed: { id: number, title: string, category: string, tags: string[], timestamp: string }[], serverTime: string } };

export interface UserProfile {
    username: string;
    avatarUrl?: string;
    notifications: number;
}
export interface Banner {
    id: string;
    title: string;
    link: string;
    variant: "Primary" | "Secondary" | "Alert";
}
export interface FeedItem {
    id: number;
    title: string;
    category: string;
    tags: string[];
    timestamp: string;
}