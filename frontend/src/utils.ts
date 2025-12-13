/**
 * A function for fetching data from the backend.
 * @param {string} URL - the URL to fetch data from
 */
export const fetcher = async (URL: string) => {
  const response = await fetch(URL, { credentials: 'include' });

  if (response.ok) {
    return response.json();
  }

  throw new Error(await response.text());
};

/**
 * A function for logging the user into the system.
 * @param URL - the backend URL
 * @param userID - the user's JumpSeller ID
 */
export const login = async (URL: string, userID: number) => {
  const response = await fetch(`${URL}/login?id=${userID}`, {
    credentials: 'include',
  });

  if (!response.ok) {
    throw new Error(await response.text());
  }
};

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
