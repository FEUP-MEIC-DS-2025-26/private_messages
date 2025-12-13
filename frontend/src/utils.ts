/**
 * Formats a date as a string.
 * @param date - the date to format
 * @returns a string representing the date
 */
export function formatDate(date: Date): string {
  const components: string[] = [];

  // compute the days elapsed since the message was sent
  const elapsedDays = (Date.now() - date.getTime()) / 86_400_000;

  if (elapsedDays == 1) {
    components.push('Yesterday');
  } else if (elapsedDays > 1) {
    const day = date.getDate();
    const month = date.getMonth() + 1; // month is 0-based
    const year = date.getFullYear();

    components.push(
      `${String(day).padStart(2, '0')}/${String(month).padStart(2, '0')}/${year}`,
    );
  }

  // format the hour and minutes
  components.push(
    `${date.getHours()}:${String(date.getMinutes()).padStart(2, '0')}`,
  );

  return components.join(' ');
}

/**
 * A function for fetching data from the backend.
 * @param {string} URL - the URL
 */
export const fetcher = (URL: string) =>
  fetch(URL, { credentials: 'include' }).then(async (res) => {
    if (res.ok) {
      return res.json();
    }
    throw new Error(await res.text());
  });

export const login = async (URL: string, userID: number) => await fetch(`${URL}/login?id=${userID}`, { credentials: 'include' }).then(async (res) => {
    if (!res.ok) {
      throw new Error(await res.text());
    }
  });
