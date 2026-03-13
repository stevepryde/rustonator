export type SpriteDirection = "up" | "down" | "left" | "right";

export function resolveDirectionalAnimationKey(
  imageKey: string,
  direction: SpriteDirection
): string {
  if (imageKey === "mob1" && (direction === "up" || direction === "down")) {
    return "mob1_updown";
  }

  return `${imageKey}_${direction}`;
}
