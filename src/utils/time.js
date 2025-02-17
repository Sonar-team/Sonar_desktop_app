// src/utils/dateUtils.js

export function getCurrentDate() {
  const now = new Date();
  return `${now.getFullYear()}${padZero(now.getMonth() + 1)}${
    padZero(now.getDate())
  }`;
}

export function padZero(value) {
  return value < 10 ? `0${value}` : value;
}
