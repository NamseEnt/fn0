export default async (req: Request): Promise<Response> => {
  const text = await req.text();
  return new Response(`received text: ${text}`, {
    status: 200,
  });
};
