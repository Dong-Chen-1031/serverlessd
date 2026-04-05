export default {
  async fetch() {
    await fetch("https://httpbin.org/delay/5");
    await fetch("https://httpbin.org/delay/5");
    return await fetch("https://httpbin.org/delay/5");
  },
};
